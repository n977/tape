use crate::buf::iter::{Frame, FrameMut, Frames, FramesMut};
use crate::buf::{Buf, BufMut, Spec};
use cpal::Sample;

pub struct Seq<T> {
    buf: Vec<T>,
    spec: Spec,
    pos: usize,
    len: usize,
}

impl<T> Seq<T>
where
    T: Sample,
{
    pub fn with_spec(spec: Spec) -> Self {
        let frames = spec.frames();
        let channels = spec.channels();
        let buf = vec![T::EQUILIBRIUM; frames * channels];

        Self {
            buf,
            spec,
            pos: 0,
            len: 0,
        }
    }
}

impl<T> Buf for Seq<T> {
    type Item = T;

    fn spec(&self) -> Spec {
        self.spec
    }

    fn frame(&self, n: usize) -> Frame<'_, T> {
        let frames = self.spec().frames();
        let channels = self.spec().channels();
        assert!(n <= frames);
        let mut frame = Vec::with_capacity(channels);

        for channel in self.buf.chunks(frames) {
            frame.push(unsafe { channel.as_ptr().cast_mut().add(n) })
        }

        frame.into()
    }

    fn frames(&self) -> Frames<'_, T> {
        Frames::new(self, 1)
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
        self.pos = std::cmp::min(n, self.len);
    }

    fn set_len(&mut self, n: usize) {
        self.len = std::cmp::min(n, self.spec().frames());
    }
}

impl<T> BufMut for Seq<T> {
    fn frame_mut(&mut self, n: usize) -> FrameMut<'_, Self::Item> {
        let frames = self.spec().frames();
        let channels = self.spec().channels();
        assert!(n <= frames);
        let mut frame = Vec::with_capacity(channels);

        for channel in self.buf.chunks_mut(frames) {
            frame.push(unsafe { channel.as_mut_ptr().add(n) })
        }

        frame.into()
    }

    fn frames_mut(&mut self) -> FramesMut<'_, Self::Item> {
        FramesMut::new(self, 1)
    }
}
