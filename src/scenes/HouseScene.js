import { BaseScene } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';

export class HouseScene extends BaseScene {
  constructor() {
    super({ key: 'HouseScene', biomeId: 'house' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1500);
    this.cameras.main.setBackgroundColor('#1a1208');

    // Warm interior parallax
    this.setupParallax([
      { key: 'particle', speed: 0, y: 0, depth: -4, tint: 0x3a2a10, alpha: 0.8 },
      { key: 'particle', speed: 0.05, y: 50, depth: -3, tint: 0x5a3a18, alpha: 0.6 },
      { key: 'particle', speed: 0.1, y: 200, depth: -2, tint: 0x8a6a3a, alpha: 0.4 },
      { key: 'particle', speed: 0.2, y: 350, depth: -1, tint: 0x4a3018, alpha: 0.5 }
    ]);

    // Warm window light
    const light = this.add.rectangle(600, 150, 200, 300, 0xffaa44, 0.08)
      .setDepth(0).setScrollFactor(0);
    this.tweens.add({
      targets: light, alpha: { from: 0.06, to: 0.1 },
      duration: 3000, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
    });

    // Dust particles
    this.setupEmitter('particle', {
      y: { min: 50, max: 400 },
      lifespan: 6000,
      speed: { min: 2, max: 6 },
      scale: { start: 0.3, end: 0 },
      alpha: { start: 0.25, end: 0 },
      frequency: 600,
      blendMode: 'ADD',
      tint: 0xffeedd,
      depth: 1
    });

    audioManager.playAmbientHouse();

    const player = this.createPlayer(200, 300);

    // Intro dialogue sequence
    this.time.delayedCall(800, () => {
      this.showDialogue('Narrator',
        'River sits at home, staring at Nadia\'s empty bed. Their sibling has fallen sick with a strange disease the doctors call "illiteracy".',
        null, 'portrait-river'
      );

      this.time.delayedCall(100, () => {
        const check = this.time.addEvent({
          delay: 100,
          callback: () => {
            if (!this.dialogueActive) {
              check.remove();
              this.intro2();
            }
          },
          repeat: -1
        });
      });
    });
  }

  intro2() {
    this.showDialogue('Grandparent',
      'River, come here. I have something for you. This is our Family Heirloom. A book that might hold the key to curing Nadia.',
      null, 'portrait-river'
    );

    const check = this.time.addEvent({
      delay: 100,
      callback: () => {
        if (!this.dialogueActive) {
          check.remove();
          this.intro3();
        }
      },
      repeat: -1
    });
  }

  intro3() {
    this.showDialogue('River',
      '...I\'ll try anything to help Nadia.',
      [{ text: 'Open the book', callback: () => this.openBook() }],
      'portrait-river'
    );
  }

  openBook() {
    // Dramatic screen flash
    audioManager.playScreenFlash();
    this.cameras.main.flash(800, 255, 255, 200);
    this.cameras.main.shake(400, 0.015);

    this.time.delayedCall(1200, () => {
      this.cameras.main.flash(600, 255, 255, 255);
      this.time.delayedCall(800, () => {
        this.transitionTo('CandyLandScene', 2000, 0xffffff);
      });
    });
  }

  update() {
    this.handleMovement(150);
    this.updateParallax();
  }
}
