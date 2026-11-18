//! Sound. Two sources, layered:
//!
//! * SFX are real mastered CC0 samples (Kenney interface / impact / RPG audio)
//!   emitted into `assets/game/audio/` by `assets/tools/build.py`. These are
//!   loaded at runtime and fall back to a synthesized blip if a file is
//!   missing, so a half-generated asset tree still boots and still makes noise.
//!
//! * Ambience is still synthesized, because no CC0 music pack is vendored. The
//!   pads are built from partials quantized to a multiple of the loop period,
//!   which makes the 4s loop mathematically seamless - no click at the wrap.

use macroquad::audio::{load_sound, load_sound_from_bytes, Sound};

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

/// Cheap deterministic pseudo-random in [-0.5,0.5) from an index.
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

/// Seamless loop pad. Every partial is snapped to a whole multiple of the loop
/// rate (1/dur Hz), so the waveform is exactly periodic over `dur` and the loop
/// point cannot click. `air` mixes in band-limited noise partials for texture
/// (wind, water, room tone).
fn pad(freqs: &[f32], detune: f32, dur: f32, air: f32, lfo_rate: f32, seed: u64) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * dur) as usize;
    let unit = 1.0 / dur; // Hz quantum that keeps the loop periodic
    let snap = |f: f32| (f / unit).round().max(1.0) * unit;

    // partials: the chord, plus a couple of harmonics for warmth
    let mut parts: Vec<(f32, f32, f32)> = Vec::new(); // (freq, amp, phase)
    for (k, f) in freqs.iter().enumerate() {
        let ph = (k as f32 * 1.37) % std::f32::consts::TAU;
        parts.push((snap(*f), 1.0, ph));
        parts.push((snap(f + k as f32 * detune), 0.55, ph * 1.7));
        parts.push((snap(f * 2.0), 0.18, ph * 2.3));
    }
    // air: 40 seeded partials spread over the top end
    let mut s = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    for _ in 0..40 {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let f = 220.0 + ((s >> 20) % 1800) as f32;
        let amp = 0.02 + ((s >> 8) % 100) as f32 / 100.0 * 0.03;
        let ph = ((s >> 33) % 628) as f32 / 100.0;
        parts.push((snap(f), amp * air, ph));
    }

    let norm = 1.0 / (freqs.len() as f32 * 1.8 + air * 2.0);
    (0..n)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let mut v = 0.0;
            for (f, a, ph) in &parts {
                v += (t * f * std::f32::consts::TAU + ph).sin() * a;
            }
            // slow breathing so the pad never sits perfectly still
            let lfo = 0.78 + 0.22 * (t * lfo_rate.min(unit * 4.0) * std::f32::consts::TAU).sin();
            v * norm * lfo * 0.5
        })
        .collect()
}

/// The samples the game actually plays. Keeping this list tight matters on the
/// web: every entry is another request before the first frame can draw.
/// Sound is loaded through the async file API, so a missing file degrades to a
/// synthesized stand-in rather than failing the boot.
pub struct Sfx {
    pub blip: Sound,
    pub good: Sound,
    pub bad: Sound,
    pub page: Sound,
    pub splash: Sound,
    pub click: Sound,
    pub select: Sound,
    pub back: Sound,
    pub question: Sound,
    pub coin: Sound,
    /// Footsteps: candy grass, aquarium sand, void stone, home wood - two
    /// variants each, alternating so a walk does not sound like one sample.
    pub steps: Vec<Sound>,
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

/// Load a real sample, or fall back to a synthesized stand-in so a missing
/// asset degrades instead of panicking.
async fn sfx(name: &str, fallback: &[f32]) -> Sound {
    let path = format!("assets/game/audio/{}.ogg", name);
    if let Ok(s) = load_sound(&path).await {
        return s;
    }
    load_sound_from_bytes(&write_wav(fallback))
        .await
        .unwrap_or_else(|e| panic!("audio subsystem rejected fallback for {}: {:?}", name, e))
}

pub async fn load() -> AudioSys {
    // --- synthesized fallbacks (also the actual ambience beds) --------------
    let blip = tone(880.0, 0.05, 0.25, 40.0);

    let mut good = tone(660.0, 0.25, 0.3, 8.0);
    let mut hi = tone(990.0, 0.3, 0.25, 8.0);
    let mut padded = vec![0.0f32; 2205];
    padded.append(&mut hi);
    good = mix(&[good, padded]);

    let bad: Vec<f32> = {
        let n = (SAMPLE_RATE as f32 * 0.35) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / SAMPLE_RATE as f32;
                let env = (-t * 6.0).min(1.0);
                let f = 110.0 + (t * 30.0).sin() * 20.0;
                ((t * f * std::f32::consts::TAU).sin() * 0.7
                    + (t * f * 4.0 * std::f32::consts::TAU).sin() * 0.3)
                    * env
                    * 0.4
            })
            .collect()
    };

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

    let splash: Vec<f32> = {
        let n = (SAMPLE_RATE as f32 * 0.6) as usize;
        (0..n)
            .map(|i| {
                let t = i as f32 / SAMPLE_RATE as f32;
                let env = (-t * 4.0).min(1.0);
                let f = (300.0 - t * 250.0).max(40.0);
                ((t * f * std::f32::consts::TAU).sin() * 0.4 + prand(i, 11) * 0.5) * env * 0.5
            })
            .collect()
    };

    let click = tone(1200.0, 0.035, 0.22, 60.0);

    // --- ambience beds: seamless, one per biome -----------------------------
    // Frequencies are snapped to the loop rate inside `pad`, so these wrap
    // without a click.
    let house = pad(&[130.8, 164.8, 196.0], 0.4, 4.0, 0.25, 0.25, 3);
    let candy = pad(&[261.6, 329.6, 392.0, 523.2], 0.7, 4.0, 0.55, 0.5, 17);
    let aqua = pad(&[98.0, 130.8, 146.8], 0.25, 4.0, 1.0, 0.25, 29);
    let cosma = pad(&[174.6, 220.0, 261.6, 349.2], 0.5, 4.0, 0.85, 0.75, 41);

    // Footsteps: grass, sand, stone, wood, then two extras for variety.
    let step_names = [
        "step_grass_a", "step_grass_b", "step_sand_a", "step_sand_b",
        "step_stone_a", "step_stone_b", "step_wood_a", "step_wood_b",
        "step_snow_a", "step_snow_b",
    ];
    let mut steps = Vec::with_capacity(step_names.len());
    for n in step_names {
        steps.push(sfx(n, &click).await);
    }

    AudioSys {
        sfx: Sfx {
            blip: sfx("blip", &blip).await,
            good: sfx("good", &good).await,
            bad: sfx("bad", &bad).await,
            page: sfx("page", &page).await,
            splash: sfx("splash", &splash).await,
            click: sfx("click", &click).await,
            select: sfx("select", &click).await,
            back: sfx("back", &click).await,
            question: sfx("question", &blip).await,
            coin: sfx("coin", &good).await,
            steps,
        },
        amb: Ambience {
            house: load_sound_from_bytes(&write_wav(&house)).await.unwrap(),
            candy: load_sound_from_bytes(&write_wav(&candy)).await.unwrap(),
            aqua: load_sound_from_bytes(&write_wav(&aqua)).await.unwrap(),
            cosma: load_sound_from_bytes(&write_wav(&cosma)).await.unwrap(),
        },
    }
}
