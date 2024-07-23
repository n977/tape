pub mod dy;
pub mod int;

use crate::buf::Spec;
use dy::DyRef;
use int::IntMut;

pub fn dy<'a, T>(buf: &'a [&'a [T]], spec: Spec) -> DyRef<'a, T> {
    DyRef::with_spec(buf, spec)
}

pub fn int_mut<T>(buf: &mut [T], spec: Spec) -> IntMut<'_, T> {
    IntMut::with_spec(buf, spec)
}
