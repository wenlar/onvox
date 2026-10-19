//! Short tones so the streamer hears the state without looking.

use std::f32::consts::TAU;
use std::sync::OnceLock;

use crate::state::Beep;

const RATE: u32 = 22050;

/// 16-bit mono PCM WAV with a 5 ms fade at both ends, so the tone does not click.
pub fn tone_wav(freq_hz: u32, ms: u32) -> Vec<u8> {
    let n = RATE * ms / 1000;
    let mut w = Vec::with_capacity(44 + n as usize * 2);
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + n * 2).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    w.extend_from_slice(&16u32.to_le_bytes());
    w.extend_from_slice(&1u16.to_le_bytes()); // PCM
    w.extend_from_slice(&1u16.to_le_bytes()); // mono
    w.extend_from_slice(&RATE.to_le_bytes());
    w.extend_from_slice(&(RATE * 2).to_le_bytes());
    w.extend_from_slice(&2u16.to_le_bytes());
    w.extend_from_slice(&16u16.to_le_bytes());
    w.extend_from_slice(b"data");
    w.extend_from_slice(&(n * 2).to_le_bytes());
    let fade = RATE as f32 * 0.005;
    for i in 0..n {
        let edge = (i.min(n - 1 - i) as f32 / fade).min(1.0);
        let v = (TAU * freq_hz as f32 * i as f32 / RATE as f32).sin() * 0.3 * edge;
        w.extend_from_slice(&((v * i16::MAX as f32) as i16).to_le_bytes());
    }
    w
}

pub fn play(b: Beep) {
    use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};
    // SND_ASYNC plays after this returns, so the sound must outlive the call.
    static ERROR: OnceLock<Vec<u8>> = OnceLock::new();
    let wav = match b {
        Beep::Error => ERROR.get_or_init(|| tone_wav(220, 300)),
    };
    unsafe { PlaySoundW(wav.as_ptr().cast(), std::ptr::null_mut(), SND_MEMORY | SND_ASYNC | SND_NODEFAULT) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_and_length() {
        let wav = tone_wav(880, 120);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        let data_len = 22050 * 120 / 1000 * 2;
        assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()) as usize, data_len);
        assert_eq!(wav.len(), 44 + data_len);
    }
}
