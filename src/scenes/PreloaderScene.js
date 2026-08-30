import Phaser from 'phaser';

export class PreloaderScene extends Phaser.Scene {
  constructor() {
    super({ key: 'PreloaderScene' });
  }

  create(data) {
    this.generateCharacterPortraits();
    this.generateUI();
    this.generateEnvironment();
    this.scene.start(data.nextScene || 'IntroScene');
  }

  // ─── Character Portraits (procedural, stylized) ──────────

  generateCharacterPortraits() {
    // River - protagonist, warm skin, determined expression
    this._drawPortrait('portrait-river', (g, w, h) => {
      // Hair (dark, short)
      g.fillStyle(0x2a1a0a, 1);
      g.fillEllipse(w / 2, h * 0.28, w * 0.55, h * 0.3);
      // Face
      g.fillStyle(0xc68642, 1);
      g.fillEllipse(w / 2, h * 0.42, w * 0.45, h * 0.42);
      // Eyes (determined, slightly narrowed)
      g.fillStyle(0xffffff, 1);
      g.fillEllipse(w * 0.38, h * 0.38, 14, 10);
      g.fillEllipse(w * 0.62, h * 0.38, 14, 10);
      g.fillStyle(0x1a0a00, 1);
      g.fillCircle(w * 0.38, h * 0.38, 5);
      g.fillCircle(w * 0.62, h * 0.38, 5);
      g.fillStyle(0xffffff, 1);
      g.fillCircle(w * 0.39, h * 0.37, 2);
      g.fillCircle(w * 0.63, h * 0.37, 2);
      // Eyebrows (slightly furrowed)
      g.lineStyle(2.5, 0x1a0a00, 1);
      g.lineBetween(w * 0.3, h * 0.32, w * 0.44, h * 0.31);
      g.lineBetween(w * 0.56, h * 0.31, w * 0.7, h * 0.32);
      // Mouth (small, determined)
      g.lineStyle(2, 0x8a4a2a, 1);
      g.lineBetween(w * 0.42, h * 0.52, w * 0.58, h * 0.52);
      // Shirt collar hint
      g.fillStyle(0x4488ff, 1);
      g.fillRect(w * 0.3, h * 0.7, w * 0.4, h * 0.3);
    });

    // Nadia - younger, hopeful, big eyes
    this._drawPortrait('portrait-nadia', (g, w, h) => {
      // Hair (longer, lighter)
      g.fillStyle(0x3a2a1a, 1);
      g.fillEllipse(w / 2, h * 0.25, w * 0.6, h * 0.35);
      g.fillRect(w * 0.15, h * 0.2, w * 0.12, h * 0.3);
      g.fillRect(w * 0.73, h * 0.2, w * 0.12, h * 0.3);
      // Face (younger, rounder)
      g.fillStyle(0xd4a574, 1);
      g.fillEllipse(w / 2, h * 0.42, w * 0.42, h * 0.4);
      // Big hopeful eyes
      g.fillStyle(0xffffff, 1);
      g.fillEllipse(w * 0.38, h * 0.38, 16, 14);
      g.fillEllipse(w * 0.62, h * 0.38, 16, 14);
      g.fillStyle(0x663300, 1);
      g.fillCircle(w * 0.38, h * 0.39, 6);
      g.fillCircle(w * 0.62, h * 0.39, 6);
      g.fillStyle(0xffffff, 1);
      g.fillCircle(w * 0.40, h * 0.37, 3);
      g.fillCircle(w * 0.64, h * 0.37, 3);
      // Gentle smile
      g.lineStyle(2, 0x9a5a3a, 1);
      g.beginPath();
      g.arc(w / 2, h * 0.50, 12, 0.1, Math.PI - 0.1, false);
      g.strokePath();
      // Dress
      g.fillStyle(0xff69b4, 1);
      g.fillRect(w * 0.25, h * 0.7, w * 0.5, h * 0.3);
    });

    // Jin - Gingerbread Man, round, friendly
    this._drawPortrait('portrait-jin', (g, w, h) => {
      // Body (gingerbread cookie shape)
      g.fillStyle(0xd2691e, 1);
      g.fillEllipse(w / 2, h * 0.45, w * 0.5, h * 0.55);
      // Head (slightly smaller)
      g.fillStyle(0xd2691e, 1);
      g.fillCircle(w / 2, h * 0.28, w * 0.22);
      // Icing smile
      g.lineStyle(3, 0xffffff, 1);
      g.beginPath();
      g.arc(w / 2, h * 0.30, 10, 0.2, Math.PI - 0.2, false);
      g.strokePath();
      // Eyes (candy buttons)
      g.fillStyle(0xffffff, 1);
      g.fillCircle(w * 0.42, h * 0.24, 5);
      g.fillCircle(w * 0.58, h * 0.24, 5);
      g.fillStyle(0x1a0a00, 1);
      g.fillCircle(w * 0.42, h * 0.24, 2.5);
      g.fillCircle(w * 0.58, h * 0.24, 2.5);
      // Icing buttons on body
      g.fillStyle(0xffffff, 1);
      g.fillCircle(w / 2, h * 0.38, 4);
      g.fillCircle(w / 2, h * 0.50, 4);
      g.fillCircle(w / 2, h * 0.62, 4);
    });

    // Kandy Kween - regal, stern
    this._drawPortrait('portrait-kandy-kween', (g, w, h) => {
      // Crown
      g.fillStyle(0xffd700, 1);
      g.fillTriangle(w * 0.25, h * 0.15, w * 0.5, h * 0.02, w * 0.75, h * 0.15);
      g.fillStyle(0xff0000, 1);
      g.fillCircle(w * 0.5, h * 0.10, 4);
      g.fillCircle(w * 0.35, h * 0.13, 3);
      g.fillCircle(w * 0.65, h * 0.13, 3);
      // Hair
      g.fillStyle(0xff1493, 1);
      g.fillEllipse(w / 2, h * 0.25, w * 0.6, h * 0.25);
      // Face
      g.fillStyle(0xffb6c1, 1);
      g.fillEllipse(w / 2, h * 0.42, w * 0.45, h * 0.4);
      // Eyes (narrow, imperious)
      g.fillStyle(0xffffff, 1);
      g.fillEllipse(w * 0.38, h * 0.38, 12, 8);
      g.fillEllipse(w * 0.62, h * 0.38, 12, 8);
      g.fillStyle(0x8b0000, 1);
      g.fillCircle(w * 0.38, h * 0.38, 4);
      g.fillCircle(w * 0.62, h * 0.38, 4);
      // Eyebrows (arched)
      g.lineStyle(2, 0x8b0000, 1);
      g.lineBetween(w * 0.30, h * 0.33, w * 0.44, h * 0.30);
      g.lineBetween(w * 0.56, h * 0.30, w * 0.70, h * 0.33);
      // Frown
      g.lineStyle(2, 0x8b0000, 1);
      g.beginPath();
      g.arc(w / 2, h * 0.56, 10, Math.PI + 0.3, -0.3, false);
      g.strokePath();
      // Necklace
      g.lineStyle(2, 0xffd700, 1);
      g.beginPath();
      g.arc(w / 2, h * 0.68, 15, 0.3, Math.PI - 0.3, false);
      g.strokePath();
      g.fillStyle(0x00ff88, 1);
      g.fillCircle(w / 2, h * 0.72, 5);
    });

    // Fire Genie - ethereal, flame-colored
    this._drawPortrait('portrait-genie', (g, w, h) => {
      // Flame body
      g.fillStyle(0xff4400, 0.8);
      g.fillTriangle(w * 0.15, h * 0.9, w * 0.5, h * 0.1, w * 0.85, h * 0.9);
      g.fillStyle(0xffaa00, 0.6);
      g.fillTriangle(w * 0.25, h * 0.85, w * 0.5, h * 0.2, w * 0.75, h * 0.85);
      g.fillStyle(0xffdd00, 0.4);
      g.fillTriangle(w * 0.35, h * 0.8, w * 0.5, h * 0.3, w * 0.65, h * 0.8);
      // Eyes
      g.fillStyle(0xffffff, 1);
      g.fillEllipse(w * 0.38, h * 0.40, 10, 8);
      g.fillEllipse(w * 0.62, h * 0.40, 10, 8);
      g.fillStyle(0xff0000, 1);
      g.fillCircle(w * 0.38, h * 0.40, 4);
      g.fillCircle(w * 0.62, h * 0.40, 4);
      // Mystical smile
      g.lineStyle(2, 0xffffff, 0.8);
      g.beginPath();
      g.arc(w / 2, h * 0.52, 10, 0.2, Math.PI - 0.2, false);
      g.strokePath();
    });
  }

  // ─── UI Assets ───────────────────────────────────────────

  generateUI() {
    const g = this.make.graphics({ x: 0, y: 0, add: false });

    // Player sprite (stylized character)
    g.clear();
    // Body
    g.fillStyle(0x4488ff, 1);
    g.fillRoundedRect(4, 16, 24, 28, 4);
    // Head
    g.fillStyle(0xc68642, 1);
    g.fillCircle(16, 10, 10);
    // Hair
    g.fillStyle(0x2a1a0a, 1);
    g.fillEllipse(16, 6, 16, 8);
    // Eyes
    g.fillStyle(0xffffff, 1);
    g.fillCircle(13, 10, 2.5);
    g.fillCircle(19, 10, 2.5);
    g.fillStyle(0x1a0a00, 1);
    g.fillCircle(13, 10, 1.2);
    g.fillCircle(19, 10, 1.2);
    g.generateTexture('player', 32, 48);
    g.clear();

    // NPC base (green body, friendly)
    g.fillStyle(0x44aa44, 1);
    g.fillRoundedRect(4, 16, 24, 28, 4);
    g.fillStyle(0xd2691e, 1);
    g.fillCircle(16, 10, 10);
    g.fillStyle(0xffffff, 1);
    g.fillCircle(13, 10, 2.5);
    g.fillCircle(19, 10, 2.5);
    g.fillStyle(0x1a0a00, 1);
    g.fillCircle(13, 10, 1.2);
    g.fillCircle(19, 10, 1.2);
    g.generateTexture('npc', 32, 48);
    g.clear();

    g.destroy();
  }

  // ─── Environment Textures ────────────────────────────────

  generateEnvironment() {
    const g = this.make.graphics({ x: 0, y: 0, add: false });

    // Particles
    g.fillStyle(0xffffff, 1);
    g.fillCircle(3, 3, 3);
    g.generateTexture('particle', 6, 6);
    g.clear();

    g.fillStyle(0xff69b4, 1);
    g.fillCircle(4, 4, 4);
    g.generateTexture('sparkle-pink', 8, 8);
    g.clear();

    g.fillStyle(0xffdd00, 1);
    g.fillCircle(4, 4, 4);
    g.generateTexture('sparkle-gold', 8, 8);
    g.clear();

    g.fillStyle(0x44aaff, 1);
    g.fillCircle(5, 5, 5);
    g.generateTexture('bubble', 10, 10);
    g.clear();

    // Stone slab
    g.fillStyle(0x888888, 1);
    g.fillRoundedRect(0, 0, 64, 64, 4);
    g.lineStyle(2, 0x555555, 1);
    g.strokeRoundedRect(0, 0, 64, 64, 4);
    g.generateTexture('stone-slab', 64, 64);
    g.clear();

    // Torch unlit
    g.fillStyle(0x8B4513, 1);
    g.fillRect(6, 8, 4, 24);
    g.fillStyle(0x555555, 1);
    g.fillRect(4, 28, 8, 4);
    g.generateTexture('torch-unlit', 16, 32);
    g.clear();

    // Torch lit
    g.fillStyle(0x8B4513, 1);
    g.fillRect(6, 8, 4, 24);
    g.fillStyle(0x555555, 1);
    g.fillRect(4, 28, 8, 4);
    g.fillStyle(0xff6600, 1);
    g.fillCircle(8, 4, 6);
    g.fillStyle(0xffaa00, 1);
    g.fillCircle(8, 3, 3);
    g.generateTexture('torch-lit', 16, 32);
    g.clear();

    g.destroy();
  }

  _drawPortrait(key, drawFn) {
    const w = 120, h = 120;
    const g = this.make.graphics({ x: 0, y: 0, add: false });
    // Background circle
    g.fillStyle(0x1a1a2e, 1);
    g.fillCircle(w / 2, h / 2, w / 2);
    g.lineStyle(2, 0xff69b4, 0.4);
    g.strokeCircle(w / 2, h / 2, w / 2 - 1);
    drawFn(g, w, h);
    g.generateTexture(key, w, h);
    g.destroy();
  }
}
