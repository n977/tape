pub mod iter;
pub mod proxy;
pub mod seq;

pub use seq::Seq;

use iter::{Frame, FrameMut, Frames, FramesMut};

pub trait Buf {
    type Item;

    fn spec(&self) -> Spec;

    fn frame(&self, n: usize) -> Frame<'_, Self::Item>;

    fn frames(&self) -> Frames<'_, Self::Item>;

    fn pos(&self) -> usize;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool;

    fn set_pos(&mut self, n: usize);

    fn set_len(&mut self, n: usize);
}

pub trait BufMut: Buf {
    fn frame_mut(&mut self, n: usize) -> FrameMut<'_, Self::Item>;

    fn frames_mut(&mut self) -> FramesMut<'_, Self::Item>;
}

#[derive(Clone, Copy)]
pub struct Spec {
    frames: usize,
    channels: usize,
}

impl Spec {
    pub fn new(frames: usize, channels: usize) -> Self {
        assert!(frames > 0);
        assert!(channels > 0);
        Self { frames, channels }
    }

    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn channels(&self) -> usize {
        self.channels
    }
}
