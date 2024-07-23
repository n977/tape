use crate::buf::iter::{Frame, Frames};
use crate::buf::{Buf, Spec};

pub struct DyRef<'a, T> {
    buf: &'a [&'a [T]],
    spec: Spec,
    pos: usize,
    len: usize,
}

impl<'a, T> DyRef<'a, T> {
    pub fn with_spec(buf: &'a [&'a [T]], spec: Spec) -> Self {
        Self {
            buf,
            spec,
            pos: 0,
            len: spec.frames(),
        }
    }
}

impl<'a, T> Buf for DyRef<'a, T> {
    type Item = T;

    fn spec(&self) -> Spec {
        self.spec
    }

    fn frame(&self, n: usize) -> Frame<'_, Self::Item> {
        let frames = self.spec().frames();
        let channels = self.spec().channels();
        assert!(n <= frames);
        let mut frame = Vec::with_capacity(channels);
        for channel in self.buf {
            unsafe {
                frame.push(channel.as_ptr().cast_mut().add(n));
            }
        }
        frame.into()
    }

    fn frames(&self) -> Frames<'_, Self::Item> {
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
