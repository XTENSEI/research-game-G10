import { BaseScene } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';

export class AquariusScene extends BaseScene {
  constructor() {
    super({ key: 'AquariusScene', biomeId: 'aquarius' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1500);
    this.cameras.main.setBackgroundColor('#050a1a');

    // Deep ocean parallax
    this.setupParallax([
      { key: 'particle', speed: 0.01, y: 0, depth: -5, tint: 0x0a1a3a, alpha: 0.6 },
      { key: 'particle', speed: 0.05, y: 80, depth: -4, tint: 0x1a3a5a, alpha: 0.4 },
      { key: 'particle', speed: 0.1, y: 200, depth: -3, tint: 0x228B22, alpha: 0.3 },
      { key: 'particle', speed: 0.2, y: 320, depth: -2, tint: 0x4682B4, alpha: 0.35 },
      { key: 'particle', speed: 0.4, y: 420, depth: -1, tint: 0xf4a460, alpha: 0.25 }
    ]);

    const player = this.createPlayer(100, 380);

    // NPCs
    this.createNPC(350, 380, 'gillbert', 'Gillbert', 'portrait-jin');
    this.createNPC(420, 380, 'scalleon', 'Scalleon', 'portrait-jin');

    // Light rays
    for (let i = 0; i < 3; i++) {
      const ray = this.add.rectangle(200 + i * 300, 0, 40, 600, 0x88ccff, 0.04)
        .setDepth(0).setScrollFactor(0).setAngle(10 + i * 5);
      this.tweens.add({
        targets: ray, x: ray.x + 30, alpha: { from: 0.03, to: 0.06 },
        duration: 4000 + i * 1000, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
      });
    }

    // Rising bubbles
    this.setupEmitter('bubble', {
      y: { min: 576, max: 576 },
      lifespan: 6000,
      speed: { min: 20, max: 50 },
      scale: { start: 0.2, end: 0.6 },
      alpha: { start: 0.4, end: 0.05 },
      frequency: 120,
      blendMode: 'ADD',
      tint: 0x88ccff,
      depth: 12
    });

    // Fish particles (small dots moving sideways)
    this.setupEmitter('particle', {
      x: { min: -50, max: -50 },
      y: { min: 100, max: 400 },
      lifespan: 8000,
      speed: { min: 30, max: 60 },
      angle: { min: 170, max: 190 },
      scale: { start: 0.3, end: 0.15 },
      alpha: { start: 0.5, end: 0.1 },
      frequency: 800,
      tint: [0xffaa44, 0x44aaff, 0xff6688],
      depth: 4
    });

    audioManager.playAmbientAquarius();

    this.gillbertTalked = false;
    const gillbert = this.npcGroup.getChildren().find(n => n.npcKey === 'gillbert');
    this.physics.add.overlap(player, gillbert, () => {
      if (!this.gillbertTalked && !this.dialogueActive) {
        this.gillbertTalked = true;
        this.startDialogue();
      }
    });
  }

  startDialogue() {
    this.showDialogue('Gillbert',
      'Hey, do you know where wild sea berries grow?',
      [
        { text: 'West?', callback: () => this.replyWest() },
        { text: 'North?', callback: () => this.replyNorth() }
      ],
      'portrait-jin'
    );
  }

  replyWest() {
    this.showDialogue('Scalleon',
      "Hmm, I believe it's in the west, I think? The locals claim so at least.",
      [{ text: 'Continue', callback: () => this.caveIntro() }],
      'portrait-jin'
    );
  }

  replyNorth() {
    this.showDialogue('Scalleon',
      "No, real ones grow from the north. The market lady told me. But the locals claim west.",
      [{ text: 'Continue', callback: () => this.caveIntro() }],
      'portrait-jin'
    );
  }

  caveIntro() {
    this.showDialogue('Narrator',
      'River reaches the Grave Cave Maze of Craze. A tablet with directional riddles guards the path deeper into Aquarius.',
      [{ text: 'Enter the cave', callback: () => this.caveMaze() }],
      'portrait-river'
    );
  }

  caveMaze() {
    // Simplified cave maze for prototype
    this.showDialogue('Tablet',
      'I am a red needle for the path you take. Which way do I point?',
      [
        { text: 'North', callback: () => this.caveCorrect() },
        { text: 'South', callback: () => this.caveWrong() },
        { text: 'East', callback: () => this.caveWrong() }
      ]
    );
  }

  caveCorrect() {
    audioManager.playCorrect();
    this.cameras.main.flash(200, 100, 200, 255);
    this.showDialogue('Narrator',
      'The path opens. River finds an ingredient for the elixir in the depths of the cave.',
      [{ text: 'Continue', callback: () => this.transitionTo('CosmaScene', 1500, 0x050510) }],
      'portrait-river'
    );
  }

  caveWrong() {
    audioManager.playWrong();
    this.cameras.main.shake(200, 0.01);
    this.showDialogue('Narrator',
      'Wrong path. The cave shifts and River is sent back to the tablet.',
      [{ text: 'Try again', callback: () => this.caveMaze() }],
      'portrait-river'
    );
  }

  update() {
    this.handleMovement(160);
    this.updateParallax();
  }
}
