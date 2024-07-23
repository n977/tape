use crate::buf::{Buf, BufMut};
use cpal::{FromSample, Sample};

pub trait Seek {
    fn seek(&mut self, t: u64) -> bool;

    fn rewind(&mut self) -> bool {
        self.seek(0)
    }
}

pub trait Write {
    type Item: Sample;

    fn write<U>(&mut self, dst: &mut U) -> usize
    where
        U: BufMut,
        U::Item: Sample + FromSample<Self::Item>;

    fn write_all<U>(&mut self, dst: &mut U)
    where
        U: BufMut,
        U::Item: Sample + FromSample<Self::Item>,
    {
        while dst.len() < dst.spec().frames() {
            self.write(dst);
        }
    }
}

impl<T> Write for T
where
    T: Buf,
    T::Item: Sample,
{
    type Item = <Self as Buf>::Item;

    fn write<U>(&mut self, dst: &mut U) -> usize
    where
        U: BufMut,
        U::Item: Sample + FromSample<Self::Item>,
    {
        let p1 = self.pos();
        let p2 = dst.len();
        let mut n = 0;

        for (src, mut dst) in self.frames().skip(p1).zip(dst.frames_mut().skip(p2)) {
            for (src, dst) in src.iter().zip(dst.iter_mut()) {
                *dst = U::Item::from_sample(*src);
            }
            n += 1;
        }

        self.set_pos(p1 + n);
        dst.set_len(p2 + n);

        n
    }
}
