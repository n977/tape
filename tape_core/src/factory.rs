use crate::buf::BufMut;
use crate::io::{Seek, Write};
use cpal::{FromSample, Sample};
use parking_lot::{Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct Factory<T> {
    items: Mutex<Vec<T>>,
    state: Mutex<FactoryState>,
    pos: AtomicUsize,
}

impl<T> Factory<T> {
    pub fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
            state: Mutex::new(FactoryState::default()),
            pos: AtomicUsize::new(0),
        }
    }

    fn items(&self) -> MutexGuard<Vec<T>> {
        self.items.lock()
    }

    pub fn map<F>(&self, f: F)
    where
        F: FnOnce(&mut Vec<T>),
    {
        f(&mut *self.items())
    }

    pub fn state(&self) -> MutexGuard<FactoryState> {
        self.state.lock()
    }

    pub fn pos(&self) -> usize {
        self.pos.load(Ordering::SeqCst)
    }
}

impl<T> Factory<T>
where
    T: Seek,
{
    pub fn seek(&self, ts: u64) {
        let flag = {
            let pos = self.pos();

            match self.items().get_mut(pos) {
                Some(item) => item.seek(ts),
                None => return,
            }
        };

        if flag {
            self.translate(1, TranslateBehavior::Modal);
        }
    }

    pub fn select(&self, pos: usize) -> bool {
        if let Some(item) = self.items().get_mut(pos) {
            if !item.rewind() {
                return false;
            }

            self.pos.store(pos, Ordering::SeqCst);

            return true;
        }

        false
    }

    pub fn translate(&self, delta: isize, behavior: TranslateBehavior) -> bool {
        if self.items().is_empty() {
            return false;
        }

        let repeat_mode = match behavior {
            TranslateBehavior::Free => RepeatMode::Playlist,
            TranslateBehavior::Modal if self.can_translate(delta) => {
                self.state().repeat_mode().get()
            }
            _ => return false,
        };
        let len = self.items().len();
        let pos = self.pos();
        let pos = match repeat_mode {
            RepeatMode::Disabled => pos.saturating_add_signed(delta),
            RepeatMode::Track => pos,
            RepeatMode::Playlist => pos.wrapping_add_signed(delta) % len,
        };

        self.select(pos)
    }

    pub fn can_translate(&self, delta: isize) -> bool {
        let repeat_mode = self.state().repeat_mode().get();
        let pos = self.pos();
        let len = self.items().len();

        !matches!(repeat_mode, RepeatMode::Disabled)
            || pos > 0 && pos != len - 1
            || pos < len && pos != 0
            || pos == 0 && delta > 0
            || pos == len - 1 && delta < 0
    }
}

impl<T> Write for Arc<Factory<T>>
where
    T: Write<Item = f32> + Seek,
{
    type Item = f32;

    fn write<U>(&mut self, buf: &mut U) -> usize
    where
        U: BufMut,
        U::Item: Sample + FromSample<Self::Item>,
    {
        let pos = self.pos();
        let mut n = 0;

        if let Some(item) = self.items().get_mut(pos) {
            n = item.write(buf);
        }

        if n == 0 {
            self.translate(1, TranslateBehavior::Modal);
        }

        n
    }
}

impl<T> Default for Factory<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub struct FactoryState {
    repeat_mode: RepeatMode,
}

impl FactoryState {
    pub fn repeat_mode(&mut self) -> &mut RepeatMode {
        &mut self.repeat_mode
    }

    pub fn replace(&mut self, src: FactoryState) -> Self {
        std::mem::replace(self, src)
    }
}

impl Default for FactoryState {
    fn default() -> Self {
        Self {
            repeat_mode: RepeatMode::Disabled,
        }
    }
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "kebab-case"))]
pub enum RepeatMode {
    Disabled,
    Track,
    Playlist,
}

impl RepeatMode {
    pub fn get(&self) -> Self {
        *self
    }

    pub fn set(&mut self, repeat_mode: Self) {
        *self = repeat_mode;
    }
}

#[derive(Clone, Copy)]
pub enum TranslateBehavior {
    Free,
    Modal,
}
