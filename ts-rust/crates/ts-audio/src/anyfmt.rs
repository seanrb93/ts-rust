//! Any of the common audio formats (MP3, Ogg Vorbis, FLAC, AAC and ALAC in M4A, WAV, ...) read as interleaved stereo `f32`, packet by packet, with the sample rate of the file.

use std::fs::File;
use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error;
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// The extensions of the files that are read (an `.ogg` may also be Ogg Opus, which `OpusReader` reads).
pub const EXTENSIONS: &[&str] = &["opus", "ogg", "oga", "mp3", "flac", "m4a", "aac", "mp4", "wav"];

pub struct AnyReader {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track: u32,
    /// the sample rate of the file
    pub rate: u32,
}

impl AnyReader {
    pub fn open(path: &Path) -> Result<AnyReader, String> {
        let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let fmt = FormatOptions { enable_gapless: true, ..Default::default() };
        let probed = symphonia::default::get_probe().format(&hint, mss, &fmt, &MetadataOptions::default()).map_err(|e| format!("{}: not an audio file that can be read ({e})", path.display()))?;
        let format = probed.format;
        let track = format.default_track().ok_or_else(|| format!("{}: no audio in the file", path.display()))?;
        let rate = track.codec_params.sample_rate.ok_or_else(|| format!("{}: the sample rate is not known", path.display()))?;
        let id = track.id;
        let decoder = symphonia::default::get_codecs().make(&track.codec_params, &DecoderOptions::default()).map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(AnyReader { format, decoder, track: id, rate })
    }

    /// The next block as interleaved stereo (a mono file is doubled, more than two channels are cut to the first two), None at the end.
    pub fn next_block(&mut self) -> Option<Vec<f32>> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(p) => p,
                Err(Error::ResetRequired) => return None,
                Err(_) => return None,
            };
            if packet.track_id() != self.track {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(d) => d,
                // (a damaged packet is skipped)
                Err(Error::DecodeError(_)) | Err(Error::IoError(_)) => continue,
                Err(_) => return None,
            };
            let spec = *decoded.spec();
            let channels = spec.channels.count().max(1);
            let mut buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
            buf.copy_interleaved_ref(decoded);
            let samples = buf.samples();
            let mut out = Vec::with_capacity(samples.len() / channels * 2);
            for frame in samples.chunks_exact(channels) {
                out.push(frame[0]);
                out.push(if channels > 1 { frame[1] } else { frame[0] });
            }
            if !out.is_empty() {
                return Some(out);
            }
        }
    }
}
