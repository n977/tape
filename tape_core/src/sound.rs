use crate::buf::{Buf, BufMut, Seq, Spec};
use crate::io::{Seek, Write};
use cpal::{FromSample, Sample};
use symphonia::audio::{AudioBuffer, AudioBufferRef};
use symphonia::codecs::{CodecParameters, Decoder, DecoderOptions};
use symphonia::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};
use symphonia::meta::MetadataOptions;
use symphonia::probe::{Hint, ProbeResult};
use symphonia::units::Time;
use thiserror::Error;

pub struct Sound {
    reader: SoundReader,
    buf: Seq<f32>,
}

impl Sound {
    pub fn new<T>(source: T) -> Result<Self, SoundError>
    where
        T: 'static + MediaSource,
    {
        let mut reader = SoundReader::new(source)?;

        if !reader.advance() {
            return Err(SoundError::Unsupported);
        }

        let src = reader.decoder.last_decoded();
        let frames = src.capacity();
        let channels = src.spec().channels.count();
        let spec = Spec::new(frames, channels);
        let mut buf = Seq::with_spec(spec);
        export_data(&src, &mut buf)?;

        let sound = Self { reader, buf };

        Ok(sound)
    }
}

impl Write for Sound {
    type Item = f32;

    fn write<U>(&mut self, dst: &mut U) -> usize
    where
        U: BufMut,
        U::Item: Sample + FromSample<Self::Item>,
    {
        if self.buf.is_empty() {
            if !self.reader.advance() {
                return 0;
            }

            let src = self.reader.decoder.last_decoded();

            if export_data(&src, &mut self.buf).is_err() {
                return 0;
            };
        }

        let p1 = self.buf.pos();
        let p2 = dst.len();
        let mut n = 0;

        for (src, mut dst) in self.buf.frames().skip(p1).zip(dst.frames_mut().skip(p2)) {
            for (src, dst) in src.iter().zip(dst.iter_mut()) {
                *dst = U::Item::from_sample(*src);
            }

            n += 1;
        }

        self.buf.set_pos(p1 + n);
        dst.set_len(p2 + n);

        n
    }
}

impl Seek for Sound {
    fn seek(&mut self, t: u64) -> bool {
        let seek = self.reader.demuxer.seek(
            SeekMode::Accurate,
            SeekTo::Time {
                time: Time {
                    seconds: t,
                    frac: 0.0,
                },
                track_id: None,
            },
        );

        seek.is_ok()
    }
}

struct SoundReader {
    demuxer: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
}

impl SoundReader {
    fn new<T>(source: T) -> Result<Self, SoundError>
    where
        T: 'static + MediaSource,
    {
        let probe = make_probe(source)?;
        let demuxer = probe.format;
        let track = demuxer.default_track().ok_or(SoundError::Invalid)?;
        let decoder = make_decoder(&track.codec_params)?;
        let reader = Self { demuxer, decoder };

        Ok(reader)
    }

    fn advance(&mut self) -> bool {
        loop {
            let packet = match self.demuxer.next_packet() {
                Ok(packet) => packet,
                Err(_) => return false,
            };

            if let Err(e) = self.decoder.decode(&packet) {
                match e {
                    symphonia::Error::DecodeError(_) | symphonia::Error::IoError(_) => continue,
                    _ => return false,
                }
            }

            return true;
        }
    }
}

fn make_probe<T>(source: T) -> Result<ProbeResult, SoundError>
where
    T: 'static + MediaSource,
{
    let source_options = MediaSourceStreamOptions::default();
    let format_options = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_options = MetadataOptions::default();
    let hint = Hint::new();
    let source = MediaSourceStream::new(Box::new(source), source_options);

    symphonia::default::get_probe()
        .format(&hint, source, &format_options, &metadata_options)
        .map_err(|_| SoundError::Unsupported)
}

fn make_decoder(codec_params: &CodecParameters) -> Result<Box<dyn Decoder>, SoundError> {
    let decoder_options = DecoderOptions::default();

    symphonia::default::get_codecs()
        .make(codec_params, &decoder_options)
        .map_err(|_| SoundError::Unsupported)
}

fn export_data<U>(src: &AudioBufferRef, dst: &mut U) -> Result<(), SoundError>
where
    U: BufMut,
    U::Item: Sample
        + FromSample<u8>
        + FromSample<u16>
        + FromSample<u32>
        + FromSample<i8>
        + FromSample<i16>
        + FromSample<i32>
        + FromSample<f32>
        + FromSample<f64>,
{
    match src {
        AudioBufferRef::U8(src) => _export_data(src, dst),
        AudioBufferRef::U16(src) => _export_data(src, dst),
        AudioBufferRef::U32(src) => _export_data(src, dst),
        AudioBufferRef::S8(src) => _export_data(src, dst),
        AudioBufferRef::S16(src) => _export_data(src, dst),
        AudioBufferRef::S32(src) => _export_data(src, dst),
        AudioBufferRef::F32(src) => _export_data(src, dst),
        AudioBufferRef::F64(src) => _export_data(src, dst),
        _ => return Err(SoundError::Unsupported),
    };
    Ok(())
}

fn _export_data<U, T>(src: &AudioBuffer<T>, dst: &mut U)
where
    T: Sample + symphonia::sample::Sample,
    U: BufMut,
    U::Item: Sample + FromSample<T>,
{
    let frames = src.capacity();
    let channels = src.spec().channels.count();
    let spec = Spec::new(frames, channels);

    dst.set_pos(0);
    dst.set_len(0);
    crate::buf::proxy::dy(src.planes().planes(), spec).write_all(dst);
}

#[derive(Error, Debug)]
pub enum SoundError {
    #[error("unsupported media container")]
    Unsupported,
    #[error("invalid media container")]
    Invalid,
}
