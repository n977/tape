use crate::buf::iter::{Frame, FrameMut, Frames, FramesMut};
use crate::buf::{Buf, BufMut, Spec};

pub struct IntMut<'a, T> {
    buf: &'a mut [T],
    spec: Spec,
    pos: usize,
    len: usize,
}

impl<'a, T> IntMut<'a, T> {
    pub fn with_spec(buf: &'a mut [T], spec: Spec) -> Self {
        Self {
            buf,
            spec,
            pos: 0,
            len: 0,
        }
    }
}

impl<T> Buf for IntMut<'_, T> {
    type Item = T;

    fn spec(&self) -> Spec {
        self.spec
    }

    fn frame(&self, n: usize) -> Frame<'_, Self::Item> {
        let frames = self.spec().frames();
        let channels = self.spec().channels();
        assert!(n <= frames);
        let mut frame = Vec::with_capacity(channels);

        for channel in self.buf.iter().take(channels) {
            frame.push(unsafe { (channel as *const T as *mut T).add(n * channels) })
        }

        Frame::new(frame.into())
    }

    fn frames(&self) -> Frames<'_, Self::Item> {
        let channels = self.spec().channels();

        Frames::new(self, channels)
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn len(&self) -> usize {
        self.len
    }

    fn is_empty(&self) -> bool {
        self.pos == self.len
    }

    fn set_pos(&mut self, n: usize) {
        self.pos = std::cmp::min(n, self.len());
    }

    fn set_len(&mut self, n: usize) {
        self.len = std::cmp::min(n, self.spec().frames())
    }
}

impl<T> BufMut for IntMut<'_, T> {
    fn frame_mut(&mut self, n: usize) -> FrameMut<'_, Self::Item> {
        let frames = self.spec().frames();
        let channels = self.spec().channels();
        assert!(n <= frames);
        let mut frame = Vec::with_capacity(channels);

        for channel in self.buf.iter_mut().take(channels) {
            frame.push(unsafe { (channel as *mut T).add(n * channels) })
        }

        FrameMut::new(frame.into())
    }

    fn frames_mut(&mut self) -> FramesMut<'_, Self::Item> {
        let channels = self.spec().channels();

        FramesMut::new(self, channels)
    }
}
