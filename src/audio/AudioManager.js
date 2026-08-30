/**
 * AudioManager - Web Audio API ambient soundscapes + SFX
 * No external audio files needed. Everything is synthesized.
 */
export class AudioManager {
  constructor() {
    this.ctx = null;
    this.masterGain = null;
    this.ambientNodes = [];
    this.initialized = false;
  }

  init() {
    if (this.initialized) return;
    this.ctx = new (window.AudioContext || window.webkitAudioContext)();
    this.masterGain = this.ctx.createGain();
    this.masterGain.gain.value = 0.3;
    this.masterGain.connect(this.ctx.destination);
    this.initialized = true;
  }

  resume() {
    if (this.ctx && this.ctx.state === 'suspended') {
      this.ctx.resume();
    }
  }

  // ─── Ambient Soundscapes ────────────────────────────────

  stopAllAmbient() {
    this.ambientNodes.forEach(node => {
      try { node.stop(); } catch (e) {}
    });
    this.ambientNodes = [];
  }

  playAmbientHouse() {
    this.stopAllAmbient();
    // Warm hum + gentle crackle
    this._playDrone(120, 0.04, 0x00);
    this._playNoise(0.015, 800, 2000);
  }

  playAmbientCandyLand() {
    this.stopAllAmbient();
    // Bright, sparkly drone
    this._playDrone(261, 0.03, 0xff); // C4
    this._playDrone(329, 0.02, 0xff); // E4
    this._playDrone(392, 0.015, 0xff); // G4
    // Sparkle noise
    this._playNoise(0.01, 4000, 8000);
  }

  playAmbientAquarius() {
    this.stopAllAmbient();
    // Deep underwater drone
    this._playDrone(55, 0.06, 0x00); // Low A
    this._playDrone(82, 0.04, 0x00); // E2
    // Water whoosh
    this._playNoise(0.02, 200, 600);
    this._playLFO(0.15, 0.02); // Slow wave
  }

  playAmbientCosma() {
    this.stopAllAmbient();
    // Ethereal, spacey
    this._playDrone(196, 0.02, 0xff); // G3
    this._playDrone(293, 0.015, 0xff); // D4
    this._playDrone(440, 0.01, 0xff); // A4
    // Cosmic noise
    this._playNoise(0.008, 100, 400);
  }

  // ─── SFX ────────────────────────────────────────────────

  playDialogueBlip() {
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'square';
    osc.frequency.value = 600 + Math.random() * 200;
    gain.gain.value = 0.06;
    gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + 0.05);
    osc.connect(gain);
    gain.connect(this.masterGain);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.05);
  }

  playChoiceSelect() {
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.value = 880;
    gain.gain.value = 0.08;
    gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + 0.1);
    osc.connect(gain);
    gain.connect(this.masterGain);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.1);
  }

  playCorrect() {
    if (!this.ctx) return;
    [523, 659, 784].forEach((freq, i) => {
      const osc = this.ctx.createOscillator();
      const gain = this.ctx.createGain();
      osc.type = 'sine';
      osc.frequency.value = freq;
      gain.gain.value = 0.08;
      gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + 0.15 + i * 0.1);
      osc.connect(gain);
      gain.connect(this.masterGain);
      osc.start(this.ctx.currentTime + i * 0.1);
      osc.stop(this.ctx.currentTime + 0.15 + i * 0.1);
    });
  }

  playWrong() {
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sawtooth';
    osc.frequency.value = 200;
    osc.frequency.exponentialRampToValueAtTime(100, this.ctx.currentTime + 0.3);
    gain.gain.value = 0.08;
    gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + 0.3);
    osc.connect(gain);
    gain.connect(this.masterGain);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.3);
  }

  playScreenFlash() {
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.value = 1200;
    gain.gain.value = 0.05;
    gain.gain.exponentialRampToValueAtTime(0.001, this.ctx.currentTime + 0.15);
    osc.connect(gain);
    gain.connect(this.masterGain);
    osc.start();
    osc.stop(this.ctx.currentTime + 0.15);
  }

  playFootstep() {
    if (!this.ctx) return;
    const buf = this.ctx.createBuffer(1, this.ctx.sampleRate * 0.06, this.ctx.sampleRate);
    const data = buf.getChannelData(0);
    for (let i = 0; i < data.length; i++) {
      data[i] = (Math.random() * 2 - 1) * Math.exp(-i / (data.length * 0.2));
    }
    const src = this.ctx.createBufferSource();
    const gain = this.ctx.createGain();
    const filter = this.ctx.createBiquadFilter();
    src.buffer = buf;
    filter.type = 'lowpass';
    filter.frequency.value = 600 + Math.random() * 400;
    gain.gain.value = 0.04;
    src.connect(filter);
    filter.connect(gain);
    gain.connect(this.masterGain);
    src.start();
  }

  // ─── Internal helpers ───────────────────────────────────

  _playDrone(freq, vol, _unused) {
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.value = freq;
    gain.gain.value = vol;
    osc.connect(gain);
    gain.connect(this.masterGain);
    osc.start();
    this.ambientNodes.push(osc);
  }

  _playNoise(vol, lo, hi) {
    if (!this.ctx) return;
    const bufSize = this.ctx.sampleRate * 4;
    const buf = this.ctx.createBuffer(1, bufSize, this.ctx.sampleRate);
    const data = buf.getChannelData(0);
    for (let i = 0; i < bufSize; i++) data[i] = Math.random() * 2 - 1;
    const src = this.ctx.createBufferSource();
    src.buffer = buf;
    src.loop = true;
    const filter = this.ctx.createBiquadFilter();
    filter.type = 'bandpass';
    filter.frequency.value = (lo + hi) / 2;
    filter.Q.value = 0.5;
    const gain = this.ctx.createGain();
    gain.gain.value = vol;
    src.connect(filter);
    filter.connect(gain);
    gain.connect(this.masterGain);
    src.start();
    this.ambientNodes.push(src);
  }

  _playLFO(rate, depth) {
    if (!this.ctx) return;
    const osc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    osc.type = 'sine';
    osc.frequency.value = rate;
    gain.gain.value = depth;
    osc.connect(gain);
    gain.connect(this.masterGain);
    osc.start();
    this.ambientNodes.push(osc);
  }
}

// Singleton
export const audioManager = new AudioManager();
