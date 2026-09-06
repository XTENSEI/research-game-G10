//! Procedural audio. Everything synthesized, zero asset files.

use macroquad::audio::{load_sound_from_bytes, Sound};

const SAMPLE_RATE: u32 = 22050;

fn write_wav(samples: &[f32]) -> Vec<u8> {
    let n = samples.len() as u32;
    let data_len = n * 2;
    let mut b = Vec::with_capacity((44 + data_len) as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&1u16.to_le_bytes()); // mono
    b.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    b.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * 32767.0) as i16;
        b.extend_from_slice(&v.to_le_bytes());
    }
    b
}

fn tone(freq: f32, dur: f32, vol: f32, decay: f32) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * dur) as usize;
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = (-t * decay).min(1.0);
            (t * freq * 2.0 * std::f32::consts::PI).sin() * vol * env
        })
        .collect()
}

/// Cheap deterministic pseudo-random in [0,1) from an index.
fn prand(i: usize, salt: u64) -> f32 {
    let x = (i as u64).wrapping_mul(2654435761).wrapping_add(salt);
    ((x >> 16) & 0xffff) as f32 / 65536.0 - 0.5
}

fn mix(chunks: &[Vec<f32>]) -> Vec<f32> {
    let len = chunks.iter().map(|c| c.len()).max().unwrap_or(0);
    let mut out = vec![0.0f32; len];
    for c in chunks {
        for (i, s) in c.iter().enumerate() {
            out[i] += s;
        }
    }
    for s in out.iter_mut() {
        *s = s.clamp(-1.0, 1.0);
    }
    out
}

pub struct Sfx {
    #[allow(dead_code)]
    pub blip: Sound,
    pub good: Sound,
    pub bad: Sound,
    pub page: Sound,
    pub splash: Sound,
}

pub struct Ambience {
    pub candy: Sound,
    pub aqua: Sound,
    pub cosma: Sound,
    pub house: Sound,
}

pub struct AudioSys {
    pub sfx: Sfx,
    pub amb: Ambience,
}

pub async fn load() -> AudioSys {
    // Blip: short tick for dialogue typewriter.
    let blip = tone(880.0, 0.05, 0.25, 40.0);

    // Good: rising two-note chime.
    let mut good = tone(660.0, 0.25, 0.3, 8.0);
    let mut hi = tone(990.0, 0.3, 0.25, 8.0);
    let mut padded = vec![0.0f32; 2205];
    padded.append(&mut hi);
    good = mix(&[good, padded]);

    // Bad: low wobble buzz.
    let bad: Vec<f32> = {
        let n = (SAMPLE_RATE as f32 * 0.35) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / SAMPLE_RATE as f32;
                let env = (-t * 6.0).min(1.0);
                let f = 110.0 + (t * 30.0).sin() * 20.0;
                ((t * f * 2.0 * std::f32::consts::PI).sin() * 0.7
                    + (t * f * 4.0 * std::f32::consts::PI).sin() * 0.3)
                    * env
                    * 0.4
            })
            .collect()
    };

    // Page turn: filtered noise swoosh.
    let page: Vec<f32> = {
        let n = (SAMPLE_RATE as f32 * 0.25) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / SAMPLE_RATE as f32;
                let env = (-t * 10.0).min(1.0);
                ((t * 47.0).sin() * 0.5 + prand(i, 7)) * env * 0.3
            })
            .collect()
    };

    // Splash: noise burst with low sweep.
    let splash: Vec<f32> = {
        let n = (SAMPLE_RATE as f32 * 0.6) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / SAMPLE_RATE as f32;
                let env = (-t * 4.0).min(1.0);
                let f = (300.0 - t * 250.0).max(40.0);
                ((t * f * 2.0 * std::f32::consts::PI).sin() * 0.4 + prand(i, 11) * 0.5) * env * 0.5
            })
            .collect()
    };

    // Biome pads: slow layered sine chords, 4s seamless loops.
    let pad = |freqs: &[f32], detune: f32| -> Vec<f32> {
        let dur = 4.0;
        let n = (SAMPLE_RATE as f32 * dur) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / SAMPLE_RATE as f32;
                let lfo = (t * 0.25 * 2.0 * std::f32::consts::PI).sin() * 0.5 + 0.5;
                let mut s = 0.0;
                for (k, f) in freqs.iter().enumerate() {
                    s += (t * (f + k as f32 * detune) * 2.0 * std::f32::consts::PI).sin();
                }
                (s / freqs.len() as f32) * (0.25 + 0.15 * lfo)
            })
            .collect()
    };

    let house = pad(&[130.8, 164.8, 196.0], 0.4); // C major low, warm
    let candy = pad(&[261.6, 329.6, 392.0, 523.2], 0.7); // C major bright sparkle
    let aqua = pad(&[98.0, 130.8, 146.8], 0.25); // G low, deep water
    let cosma = pad(&[174.6, 220.0, 261.6, 349.2], 0.5); // F spacey

    async fn mk(b: Vec<u8>) -> Sound {
        match load_sound_from_bytes(&b).await {
            Ok(s) => s,
            Err(_) => load_sound_from_bytes(&write_wav(&[0.0; 100])).await.unwrap(),
        }
    }

    AudioSys {
        sfx: Sfx {
            blip: mk(write_wav(&blip)).await,
            good: mk(write_wav(&good)).await,
            bad: mk(write_wav(&bad)).await,
            page: mk(write_wav(&page)).await,
            splash: mk(write_wav(&splash)).await,
        },
        amb: Ambience {
            house: mk(write_wav(&house)).await,
            candy: mk(write_wav(&candy)).await,
            aqua: mk(write_wav(&aqua)).await,
            cosma: mk(write_wav(&cosma)).await,
        },
    }
}
