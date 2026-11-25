//! Audio system: CC0 RPG & underwater music tracks, Kenney SFX,
//! and character-pitched Undertale-style voice blips.

use macroquad::audio::{load_sound, load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};

const SAMPLE_RATE: u32 = 22050;

fn write_wav(samples: &[f32]) -> Vec<u8> {
    let n = samples.len() as u32;
    let data_len = n * 2;
    let mut b = Vec::with_capacity((44 + data_len) as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
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
            (t * freq * std::f32::consts::TAU).sin() * vol * env
        })
        .collect()
}

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

/// Seamless fallback ambience pad in case an `.ogg` track is absent.
fn pad(freqs: &[f32], detune: f32, dur: f32, air: f32, lfo_rate: f32, seed: u64) -> Vec<f32> {
    let n = (SAMPLE_RATE as f32 * dur) as usize;
    let unit = 1.0 / dur;
    let snap = |f: f32| (f / unit).round().max(1.0) * unit;

    let mut parts: Vec<(f32, f32, f32)> = Vec::new();
    for (k, f) in freqs.iter().enumerate() {
        let ph = (k as f32 * 1.37) % std::f32::consts::TAU;
        parts.push((snap(*f), 1.0, ph));
        parts.push((snap(f + k as f32 * detune), 0.55, ph * 1.7));
        parts.push((snap(f * 2.0), 0.18, ph * 2.3));
    }
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
            let lfo = 0.78 + 0.22 * (t * lfo_rate.min(unit * 4.0) * std::f32::consts::TAU).sin();
            v * norm * lfo * 0.45
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MusicTrack {
    Town,
    Kelp,
    Coral,
    Cave,
    Slums,
    Rainbow,
    Battle,
}

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
    pub steps: Vec<Sound>,
}

pub struct MusicTracks {
    pub town: Sound,
    pub kelp: Sound,
    pub coral: Sound,
    pub cave: Sound,
    pub slums: Sound,
    pub rainbow: Sound,
    pub battle: Sound,
}

pub struct Ambience {
    pub candy: Sound,
    pub aqua: Sound,
    pub cosma: Sound,
    pub house: Sound,
}

pub struct AudioSys {
    pub sfx: Sfx,
    pub mus: MusicTracks,
    pub amb: Ambience,
    pub blips: [Sound; 5],
    active: Option<MusicTrack>,
    pub muted: bool,
    pub music_vol: f32,
    pub sfx_vol: f32,
}

impl AudioSys {
    pub fn track_sound(&self, id: MusicTrack) -> &Sound {
        match id {
            MusicTrack::Town => &self.mus.town,
            MusicTrack::Kelp => &self.mus.kelp,
            MusicTrack::Coral => &self.mus.coral,
            MusicTrack::Cave => &self.mus.cave,
            MusicTrack::Slums => &self.mus.slums,
            MusicTrack::Rainbow => &self.mus.rainbow,
            MusicTrack::Battle => &self.mus.battle,
        }
    }

    pub fn ensure_music(&mut self, id: MusicTrack) {
        if self.active == Some(id) {
            return;
        }
        if let Some(prev) = self.active {
            stop_sound(self.track_sound(prev));
        }
        let vol = if self.muted { 0.0 } else { self.music_vol };
        play_sound(self.track_sound(id), PlaySoundParams { looped: true, volume: vol });
        self.active = Some(id);
    }

    #[allow(dead_code)]
    pub fn stop_music(&mut self) {
        if let Some(prev) = self.active.take() {
            stop_sound(self.track_sound(prev));
        }
    }

    pub fn toggle_mute(&mut self) -> bool {
        self.muted = !self.muted;
        self.apply_music_vol();
        self.muted
    }

    pub fn set_music_vol(&mut self, v: f32) {
        self.music_vol = v.clamp(0.0, 1.0);
        self.apply_music_vol();
    }

    pub fn set_sfx_vol(&mut self, v: f32) {
        self.sfx_vol = v.clamp(0.0, 1.0);
    }

    fn apply_music_vol(&self) {
        if let Some(cur) = self.active {
            let vol = if self.muted { 0.0 } else { self.music_vol };
            set_sound_volume(self.track_sound(cur), vol);
        }
    }

    pub fn play_sfx(&self, snd: &Sound, scale: f32) {
        if self.muted {
            return;
        }
        let vol = (scale * self.sfx_vol).clamp(0.0, 1.0);
        if vol > 0.005 {
            play_sound(snd, PlaySoundParams { looped: false, volume: vol });
        }
    }

    pub fn play_voice(&self, speaker: &str) {
        if self.muted {
            return;
        }
        let idx = match speaker {
            "Queen Crab" | "Tenebris" | "Tow-Tou" => 0,
            "Jack" | "Jill" | "Guard 1" | "Guard 2" | "Grandparent" | "Mark" => 1,
            "River" | "Nadia" | "Thaumiel" | "Magic Fire Genie" => 2,
            "Kenny" | "Sophie" | "Market Lady" | "Gillbert" => 3,
            _ => 4,
        };
        let vol = (0.14 * self.sfx_vol).clamp(0.0, 1.0);
        if vol > 0.005 {
            play_sound(&self.blips[idx], PlaySoundParams { looped: false, volume: vol });
        }
    }
}

async fn sfx(name: &str, fallback: &[f32]) -> Sound {
    let path = format!("assets/game/audio/{}.ogg", name);
    if let Ok(s) = load_sound(&path).await {
        return s;
    }
    load_sound_from_bytes(&write_wav(fallback))
        .await
        .unwrap_or_else(|e| panic!("audio subsystem rejected fallback for {}: {:?}", name, e))
}

async fn mus(name: &str, fallback: &[f32]) -> Sound {
    let path = format!("assets/game/audio/{}.ogg", name);
    if let Ok(s) = load_sound(&path).await {
        return s;
    }
    load_sound_from_bytes(&write_wav(fallback))
        .await
        .unwrap_or_else(|e| panic!("audio subsystem rejected fallback for {}: {:?}", name, e))
}

pub async fn load() -> AudioSys {
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

    let step_names = [
        "step_grass_a", "step_grass_b", "step_sand_a", "step_sand_b",
        "step_stone_a", "step_stone_b", "step_wood_a", "step_wood_b",
        "step_snow_a", "step_snow_b",
    ];
    let mut steps = Vec::with_capacity(step_names.len());
    for n in step_names {
        steps.push(sfx(n, &click).await);
    }

    let town_pad = pad(&[130.8, 164.8, 196.0], 0.4, 4.0, 0.25, 0.25, 3);
    let kelp_pad = pad(&[196.0, 246.9, 293.7, 392.0], 0.5, 4.0, 0.55, 0.4, 17);
    let coral_pad = pad(&[98.0, 130.8, 164.8], 0.25, 4.0, 0.85, 0.3, 29);
    let slums_pad = pad(&[110.0, 146.8, 174.6, 220.0], 0.4, 4.0, 0.90, 0.65, 41);

    let town = mus("mus_town", &town_pad).await;
    let kelp = mus("mus_kelp", &kelp_pad).await;
    let coral = mus("mus_coral", &coral_pad).await;
    let cave = mus("mus_cave", &coral_pad).await;
    let slums = mus("mus_slums", &slums_pad).await;
    let rainbow = mus("mus_rainbow", &kelp_pad).await;
    let battle = mus("mus_battle", &slums_pad).await;

    let amb = Ambience {
        house: town.clone(),
        candy: kelp.clone(),
        aqua: coral.clone(),
        cosma: slums.clone(),
    };

    let blips = [
        load_sound_from_bytes(&write_wav(&tone(210.0, 0.048, 0.22, 38.0))).await.unwrap(),
        load_sound_from_bytes(&write_wav(&tone(320.0, 0.045, 0.20, 42.0))).await.unwrap(),
        load_sound_from_bytes(&write_wav(&tone(440.0, 0.042, 0.19, 44.0))).await.unwrap(),
        load_sound_from_bytes(&write_wav(&tone(540.0, 0.040, 0.19, 46.0))).await.unwrap(),
        load_sound_from_bytes(&write_wav(&tone(660.0, 0.038, 0.18, 48.0))).await.unwrap(),
    ];

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
        mus: MusicTracks { town, kelp, coral, cave, slums, rainbow, battle },
        amb,
        blips,
        active: None,
        muted: false,
        music_vol: 0.26,
        sfx_vol: 0.55,
    }
}
