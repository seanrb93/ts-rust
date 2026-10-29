//! Audio output: opens the default device and feeds it from the native mixer (`ts-audio`); also a WAV dump for
//! headless testing.

use std::sync::Mutex;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// The sound is held (the app is in the background): the mixer does not advance and nothing is played.
pub static PAUSED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

static WAV: Mutex<Option<(String, Vec<i16>)>> = Mutex::new(None);

/// Open the default output device.  Failure is reported and the game runs silent.
pub fn start_device() {
    let host = cpal::default_host();
    let Some(device) = host.default_output_device() else {
        eprintln!("[audio] no output device; running silent");
        return;
    };
    let Ok(cfg) = device.default_output_config() else {
        eprintln!("[audio] no output config; running silent");
        return;
    };
    if cfg.sample_format() != cpal::SampleFormat::F32 {
        eprintln!("[audio] unsupported sample format {:?}; running silent", cfg.sample_format());
        return;
    }
    let rate = cfg.sample_rate().0;
    let ch = cfg.channels() as usize;
    let mut tmp: Vec<f32> = Vec::new();
    let stream = device.build_output_stream(
        &cfg.into(),
        move |data: &mut [f32], _| {
            if PAUSED.load(std::sync::atomic::Ordering::Relaxed) {
                data.fill(0.0);
                return;
            }
            let frames = data.len() / ch;
            tmp.resize(frames * 2, 0.0);
            ts_audio::render(&mut tmp, rate);
            for (i, f) in data.chunks_exact_mut(ch).enumerate() {
                f[0] = tmp[2 * i];
                if ch > 1 {
                    f[1] = tmp[2 * i + 1];
                }
                for x in f.iter_mut().skip(2) {
                    *x = 0.0;
                }
            }
        },
        |e| eprintln!("[audio] stream error: {e}"),
        None,
    );
    match stream {
        Ok(s) => {
            if s.play().is_ok() {
                eprintln!("[audio] output started ({rate} Hz, {ch} ch)");
            }
            std::mem::forget(s);
        }
        Err(e) => eprintln!("[audio] cannot open stream: {e}"),
    }
}

/// Record all audio rendered per game frame into a WAV file written by `finish_wav`.
pub fn enable_wav(path: &str) {
    *WAV.lock().unwrap() = Some((path.to_string(), Vec::new()));
}

/// Called once per presented frame in headless mode: render 1/60 s at 48 kHz into the dump.
pub fn wav_frame() {
    if let Some((_, pcm)) = WAV.lock().unwrap().as_mut() {
        let mut buf = vec![0.0f32; 800 * 2];
        ts_audio::render(&mut buf, 48000);
        if std::env::var_os("TS_AUDLOG").is_some() {
            static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            if N.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 30 == 0 {
                let peak = buf.iter().fold(0.0f32, |a, b| a.max(b.abs()));
                eprintln!("[audio] voices={} music={} peak={:.2}", ts_audio::active_voices(), ts_audio::music_playing(), peak);
            }
        }
        pcm.extend(buf.iter().map(|s| (s * 32767.0) as i16));
    }
}

pub fn finish_wav() {
    if let Some((path, pcm)) = WAV.lock().unwrap().take() {
        let mut out = Vec::with_capacity(44 + pcm.len() * 2);
        let data_len = (pcm.len() * 2) as u32;
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVEfmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&2u16.to_le_bytes());
        out.extend_from_slice(&48000u32.to_le_bytes());
        out.extend_from_slice(&(48000u32 * 4).to_le_bytes());
        out.extend_from_slice(&4u16.to_le_bytes());
        out.extend_from_slice(&16u16.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        for s in pcm {
            out.extend_from_slice(&s.to_le_bytes());
        }
        std::fs::write(&path, out).ok();
        eprintln!("[audio] wrote {path}");
    }
}
