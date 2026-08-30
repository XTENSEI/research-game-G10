import { BaseScene } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';

export class CosmaScene extends BaseScene {
  constructor() {
    super({ key: 'CosmaScene', biomeId: 'cosma' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1500);
    this.cameras.main.setBackgroundColor('#020208');

    // Deep space parallax
    this.setupParallax([
      { key: 'particle', speed: 0.005, y: 0, depth: -5, tint: 0x0a0a2a, alpha: 0.5 },
      { key: 'particle', speed: 0.03, y: 60, depth: -4, tint: 0x222244, alpha: 0.4 },
      { key: 'sparkle-gold', speed: 0.08, y: 150, depth: -3, tint: 0xffdd00, alpha: 0.3 },
      { key: 'particle', speed: 0.15, y: 300, depth: -2, tint: 0x444466, alpha: 0.3 },
      { key: 'particle', speed: 0.35, y: 430, depth: -1, tint: 0x666688, alpha: 0.2 }
    ]);

    const player = this.createPlayer(100, 400);

    // Neon star particles (multi-colored)
    this.setupEmitter('sparkle-gold', {
      y: { min: 0, max: 576 },
      lifespan: 3500,
      speed: { min: 1, max: 5 },
      scale: { start: 0.4, end: 0 },
      alpha: { start: 0.7, end: 0 },
      frequency: 180,
      blendMode: 'ADD',
      tint: [0xffdd00, 0x44aaff, 0xff4444, 0xffffff],
      depth: 13
    });

    // Floating dust
    this.setupEmitter('particle', {
      y: { min: 0, max: 576 },
      lifespan: 5000,
      speed: { min: 1, max: 3 },
      scale: { start: 0.2, end: 0 },
      alpha: { start: 0.3, end: 0 },
      frequency: 400,
      blendMode: 'ADD',
      tint: 0x8888aa,
      depth: 1
    });

    audioManager.playAmbientCosma();

    // Torches
    this.torches = [];
    for (let i = 0; i < 3; i++) {
      const torch = this.physics.add.sprite(380 + i * 130, 380, 'torch-unlit');
      torch.setScale(3);
      torch.setImmovable(true);
      torch.setDepth(10);
      torch.lit = false;
      this.torches.push(torch);
    }

    this.starsCaught = 0;
    this.starsNeeded = 15;
    this.lives = 3;
    this.starActive = false;

    this.time.delayedCall(800, () => {
      this.showDialogue('Narrator',
        'The Rainbow Road stretches before you. Three unlit torches stand at the portal. Catch the stars to light them.',
        [{ text: 'Begin', callback: () => this.startCatching() }],
        'portrait-river'
      );
    });
  }

  startCatching() {
    this.starsCaught = 0;
    this.lives = 3;
    this.starActive = true;

    this.starTimer = this.time.addEvent({
      delay: 700,
      callback: () => this.spawnStar(),
      repeat: 24
    });
  }

  spawnStar() {
    if (!this.starActive) return;
    const x = Phaser.Math.Between(80, 940);
    const isDying = Math.random() < 0.25;
    const tex = isDying ? 'particle' : 'sparkle-gold';
    const tint = isDying ? 0xff2222 : 0xffdd00;

    const star = this.physics.add.sprite(x, -15, tex);
    star.setScale(isDying ? 0.8 : 1.2);
    star.setDepth(11);
    star.isDying = isDying;
    star.setVelocityY(Phaser.Math.Between(100, 180));
    star.setTint(tint);

    // Glow pulse
    this.tweens.add({
      targets: star, alpha: { from: 0.6, to: 1 }, scale: star.scale * 1.3,
      duration: 300, yoyo: true, repeat: -1
    });

    this.physics.add.overlap(this.player, star, () => {
      if (!star.active) return;
      star.destroy();
      if (isDying) {
        this.lives--;
        audioManager.playWrong();
        this.cameras.main.shake(150, 0.012);
        this.cameras.main.flash(150, 255, 50, 50, false);
        if (this.lives <= 0) this.failCatching();
      } else {
        this.starsCaught++;
        audioManager.playCorrect();
        this.cameras.main.flash(80, 255, 220, 0, false);
        if (this.starsCaught >= this.starsNeeded) this.completeCatching();
      }
    });
  }

  failCatching() {
    this.starActive = false;
    if (this.starTimer) this.starTimer.remove();
    this.showDialogue('Narrator',
      'No lives left! The stars fade away...',
      [{ text: 'Retry', callback: () => this.startCatching() }],
      'portrait-river'
    );
  }

  completeCatching() {
    this.starActive = false;
    if (this.starTimer) this.starTimer.remove();
    audioManager.playScreenFlash();
    this.cameras.main.flash(500, 255, 220, 0);

    this.time.delayedCall(600, () => {
      this.showDialogue('Narrator',
        'The stars fuse into a poem carved on stone slabs. Place them on the torch podiums.',
        [{ text: 'Place slabs', callback: () => this.lightTorches() }],
        'portrait-river'
      );
    });
  }

  lightTorches() {
    this.torches.forEach((torch, i) => {
      this.time.delayedCall(i * 600, () => {
        torch.setTexture('torch-lit');
        torch.lit = true;
        audioManager.playScreenFlash();
        this.cameras.main.flash(150, 255, 150, 0, false);
        this.tweens.add({
          targets: torch, scaleY: 3.8, duration: 200, yoyo: true, ease: 'Bounce'
        });
      });
    });

    this.time.delayedCall(2200, () => this.summonGenie());
  }

  summonGenie() {
    audioManager.playScreenFlash();
    this.cameras.main.flash(800, 255, 100, 0);

    this.time.delayedCall(600, () => {
      this.showDialogue('Magic Fire Genie',
        'I am the Fire Genie! You have lit my torches. Now answer this: What was the meaning of the poem?',
        [
          { text: 'Impact outlasts presence', callback: () => this.genieCorrect() },
          { text: 'Stars are pretty', callback: () => this.genieWrong() }
        ],
        'portrait-genie'
      );
    });
  }

  genieCorrect() {
    audioManager.playCorrect();
    this.cameras.main.flash(500, 255, 220, 100);
    this.showDialogue('Fire Genie',
      'Correct! The poem speaks of how people shape us even after they are gone. You have earned the Gift of Education.',
      [{ text: 'Continue', callback: () => this.transitionTo('EndingScene', 2000, 0xffffff) }],
      'portrait-genie'
    );
  }

  genieWrong() {
    audioManager.playWrong();
    this.cameras.main.shake(300, 0.015);
    this.showDialogue('Fire Genie',
      'Wrong! The poem restarts with new verses...',
      [{ text: 'Retry', callback: () => this.startCatching() }],
      'portrait-genie'
    );
  }

  update() {
    this.handleMovement(180);
    this.updateParallax();
  }
}
