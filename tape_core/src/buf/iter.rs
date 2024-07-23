use crate::buf::{Buf, BufMut};
use std::marker::PhantomData;

pub struct Frames<'a, T>
where
    T: 'a,
{
    start: Box<[*mut T]>,
    end: Box<[*mut T]>,
    step: usize,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Frames<'a, T> {
    pub fn new<U>(buf: &U, step: usize) -> Self
    where
        U: Buf<Item = T>,
    {
        Self {
            start: buf.frame(0).into(),
            end: buf.frame(buf.spec().frames()).into(),
            step,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for Frames<'a, T> {
    type Item = Frame<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            return None;
        }
        let frame = Frame::new(self.start.clone());
        for ptr in self.start.iter_mut() {
            unsafe {
                *ptr = ptr.add(self.step);
            }
        }
        Some(frame)
    }
}

pub struct Frame<'a, T> {
    frame: Box<[*mut T]>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Frame<'a, T> {
    pub fn new(frame: Box<[*mut T]>) -> Self {
        Self {
            frame,
            _marker: PhantomData,
        }
    }

    pub fn into_raw(self) -> Box<[*mut T]> {
        self.frame
    }

    pub fn iter(&self) -> impl Iterator<Item = &'_ T> {
        self.frame
            .iter()
            .map(|ptr| unsafe { ptr.as_ref_unchecked() })
    }
}

impl<T> Frame<'_, T>
where
    T: Copy,
{
    pub fn into_vec(self) -> Vec<T> {
        self.iter().copied().collect::<_>()
    }
}

impl<'a, T> From<Frame<'a, T>> for Box<[*mut T]> {
    fn from(frame: Frame<'a, T>) -> Self {
        frame.into_raw()
    }
}

impl<'a, T> From<Vec<*mut T>> for Frame<'a, T> {
    fn from(frame: Vec<*mut T>) -> Self {
        Self::new(frame.into())
    }
}

pub struct FramesMut<'a, T>
where
    T: 'a,
{
    start: Box<[*mut T]>,
    end: Box<[*mut T]>,
    step: usize,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> FramesMut<'a, T> {
    pub fn new<U>(buf: &mut U, step: usize) -> Self
    where
        U: Buf<Item = T> + BufMut,
    {
        let end = unsafe {
            let frame = buf.frame_mut(buf.spec().frames());
            std::mem::transmute::<FrameMut<'_, T>, FrameMut<'_, T>>(frame)
        };
        let start = buf.frame_mut(0);
        Self {
            start: start.into(),
            end: end.into(),
            step,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for FramesMut<'a, T> {
    type Item = FrameMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }
        let frame = FrameMut::new(self.start.clone());
        for ptr in self.start.iter_mut() {
            unsafe {
                *ptr = ptr.add(self.step);
            }
        }
        Some(frame)
    }
}

pub struct FrameMut<'a, T> {
    frame: Box<[*mut T]>,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> FrameMut<'a, T> {
    pub fn new(frame: Box<[*mut T]>) -> Self {
        Self {
            frame,
            _marker: PhantomData,
        }
    }

    pub fn into_raw(self) -> Box<[*mut T]> {
        self.frame
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut T> {
        self.frame
            .iter_mut()
            .map(|ptr| unsafe { ptr.as_mut_unchecked() })
    }
}

impl<'a, T> From<FrameMut<'a, T>> for Box<[*mut T]> {
    fn from(frame: FrameMut<'a, T>) -> Self {
        frame.into_raw()
    }
}

impl<'a, T> From<Vec<*mut T>> for FrameMut<'a, T> {
    fn from(frame: Vec<*mut T>) -> Self {
        Self::new(frame.into())
    }
}
