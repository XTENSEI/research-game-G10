import Phaser from 'phaser';

export class PreloaderScene extends Phaser.Scene {
  constructor() {
    super({ key: 'PreloaderScene' });
  }

  create() {
    // Generate placeholder textures programmatically
    this.generatePlaceholders();
    this.scene.start('HouseScene');
  }

  generatePlaceholders() {
    const g = this.make.graphics({ x: 0, y: 0, add: false });

    // Player placeholder (blue rectangle)
    g.fillStyle(0x4488ff, 1);
    g.fillRect(0, 0, 32, 48);
    g.generateTexture('player', 32, 48);
    g.clear();

    // NPC placeholder (green rectangle)
    g.fillStyle(0x44cc44, 1);
    g.fillRect(0, 0, 32, 48);
    g.generateTexture('npc', 32, 48);
    g.clear();

    // Dialogue box background
    g.fillStyle(0x000000, 0.85);
    g.fillRoundedRect(0, 0, 800, 150, 12);
    g.lineStyle(2, 0xffffff, 1);
    g.strokeRoundedRect(0, 0, 800, 150, 12);
    g.generateTexture('dialogue-box', 800, 150);
    g.clear();

    // Choice button
    g.fillStyle(0x333333, 0.9);
    g.fillRoundedRect(0, 0, 300, 40, 8);
    g.lineStyle(1, 0xff69b4, 1);
    g.strokeRoundedRect(0, 0, 300, 40, 8);
    g.generateTexture('choice-btn', 300, 40);
    g.clear();

    // Choice button hover
    g.fillStyle(0xff69b4, 0.3);
    g.fillRoundedRect(0, 0, 300, 40, 8);
    g.lineStyle(2, 0xff69b4, 1);
    g.strokeRoundedRect(0, 0, 300, 40, 8);
    g.generateTexture('choice-btn-hover', 300, 40);
    g.clear();

    // Star (for Cosma puzzle)
    g.fillStyle(0xffdd00, 1);
    g.fillCircle(12, 12, 12);
    g.generateTexture('star-gold', 24, 24);
    g.clear();

    // Dying star
    g.fillStyle(0xff0000, 1);
    g.fillCircle(12, 12, 12);
    g.generateTexture('star-red', 24, 24);
    g.clear();

    // Torch (unlit)
    g.fillStyle(0x8B4513, 1);
    g.fillRect(4, 0, 8, 32);
    g.generateTexture('torch-unlit', 16, 32);
    g.clear();

    // Torch (lit)
    g.fillStyle(0x8B4513, 1);
    g.fillRect(4, 0, 8, 32);
    g.fillStyle(0xff6600, 1);
    g.fillCircle(8, -4, 8);
    g.generateTexture('torch-lit', 16, 40);
    g.clear();

    // Particle (tiny white dot)
    g.fillStyle(0xffffff, 1);
    g.fillCircle(2, 2, 2);
    g.generateTexture('particle', 4, 4);
    g.clear();

    // Bubble (for Aquarius)
    g.lineStyle(1, 0xaaddff, 0.6);
    g.strokeCircle(8, 8, 8);
    g.generateTexture('bubble', 16, 16);
    g.clear();

    // Stone slab
    g.fillStyle(0x888888, 1);
    g.fillRect(0, 0, 64, 64);
    g.lineStyle(2, 0x666666, 1);
    g.strokeRect(0, 0, 64, 64);
    g.generateTexture('stone-slab', 64, 64);
    g.clear();

    g.destroy();
  }
}
