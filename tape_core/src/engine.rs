use crate::buf::Spec;
use crate::io::Write;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BuildStreamError, Device, FromSample, SampleFormat, SizedSample, Stream, StreamConfig};
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, warn};

pub struct Engine<U> {
    provider: Arc<U>,
    device: Device,
    stream: Option<Stream>,
    state: PlaybackState,
}

impl<U> Engine<U> {
    pub fn provider(&self) -> &U {
        self.provider.as_ref()
    }

    pub fn state(&mut self) -> PlaybackStateManager<'_> {
        PlaybackStateManager {
            stream: self.stream.as_ref(),
            state: &mut self.state,
        }
    }
}

impl<U> Engine<U>
where
    Arc<U>: 'static + Write<Item = f32> + Send + Sync,
{
    pub fn new(provider: U) -> Result<Self, EngineError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(EngineError::Unsupported)?;
        let provider = Arc::new(provider);
        let engine = Self {
            provider,
            device,
            stream: None,
            state: PlaybackState::Paused,
        };

        Ok(engine)
    }

    pub fn _run<T>(&mut self, config: &StreamConfig) -> Result<(), EngineError>
    where
        T: SizedSample + FromSample<f32>,
    {
        let mut provider = self.provider.clone();
        let channels = config.channels as usize;
        let stream = self
            .device
            .build_output_stream(
                config,
                move |buf: &mut [T], _: &_| {
                    let frames = buf.len() / channels;
                    let spec = Spec::new(frames, channels);
                    let mut dst = crate::buf::proxy::int_mut(buf, spec);
                    provider.write_all(&mut dst);
                },
                |_| {},
                None,
            )
            .map_err(EngineError::ConnectionFailed)?;

        debug!(
            "Device configuration: channels: {}, sample rate: {}, sample format: {}",
            config.channels,
            config.sample_rate.0,
            std::any::type_name::<T>()
        );

        self.stream.replace(stream);

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), EngineError> {
        let config = self
            .device
            .default_output_config()
            .map_err(|_| EngineError::Unsupported)?;

        match config.sample_format() {
            SampleFormat::U8 => self._run::<u8>(&config.into()),
            SampleFormat::U16 => self._run::<u16>(&config.into()),
            SampleFormat::U32 => self._run::<u32>(&config.into()),
            SampleFormat::U64 => self._run::<u64>(&config.into()),
            SampleFormat::I8 => self._run::<i8>(&config.into()),
            SampleFormat::I16 => self._run::<i16>(&config.into()),
            SampleFormat::I32 => self._run::<i32>(&config.into()),
            SampleFormat::I64 => self._run::<i64>(&config.into()),
            SampleFormat::F32 => self._run::<f32>(&config.into()),
            SampleFormat::F64 => self._run::<f64>(&config.into()),
            _ => return Err(EngineError::Unsupported),
        }?;

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub enum PlaybackState {
    Paused,
    Playing,
}

impl PlaybackState {
    pub fn get(&self) -> Self {
        *self
    }

    pub fn set(&mut self, state: Self) {
        *self = state;
    }
}

pub struct PlaybackStateManager<'a> {
    stream: Option<&'a Stream>,
    state: &'a mut PlaybackState,
}

impl<'a> PlaybackStateManager<'a> {
    pub fn get(&self) -> PlaybackState {
        self.state.get()
    }

    pub fn set(&mut self, state: PlaybackState) {
        if let Some(stream) = self.stream {
            match &state {
                PlaybackState::Paused => {
                    if let Err(e) = stream.pause() {
                        warn!("{:#}", e);
                    }
                }
                PlaybackState::Playing => {
                    if let Err(e) = stream.play() {
                        warn!("{:#}", e);
                    }
                }
            };

            self.state.set(state);
        }
    }
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("unsupported device")]
    Unsupported,
    #[error(transparent)]
    ConnectionFailed(#[from] BuildStreamError),
}
