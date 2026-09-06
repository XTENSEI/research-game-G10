import { BaseScene, GROUND_Y } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';

export class EndingScene extends BaseScene {
  constructor() {
    super({ key: 'EndingScene', biomeId: 'house' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(800);
    this.cameras.main.setBackgroundColor('#1a0505');
    this.setTheme('#ff6b6b', '#8a1a1a');

    this.setupSky('sky-crumble');

    // Broken horizon rubble silhouettes
    this.addBand('ridge-cosma', 470, 210, -4, 0.55, 0.1);

    this.setupGround('tile-neon', GROUND_Y, 0x1a0a0a);

    // Glowing fractures creeping up the floor
    const cracks = this.add.graphics().setDepth(7).setScrollFactor(0);
    cracks.lineStyle(2, 0xff6b2a, 0.6);
    [[200, 470], [430, 470], [700, 470], [900, 470]].forEach(([x, y], i) => {
      cracks.lineBetween(x, y, x - 20 - i * 6, y - 30);
      cracks.lineBetween(x, y, x + 16 + i * 4, y - 46);
      cracks.lineBetween(x - 20 - i * 6, y - 30, x - 30, y - 12);
    });
    this.cracks = cracks;

    this.setupVignette(0.6, 89);

    const player = this.createPlayer(512, GROUND_Y - 56);

    // Crumble debris falling from the sky
    this.debris = this.setupEmitter('particle', {
      y: { min: -20, max: 120 },
      x: { min: 0, max: 1024 },
      lifespan: 2600,
      speed: { min: 60, max: 140 },
      angle: { min: 80, max: 100 },
      gravityY: 220,
      scale: { start: 0.7, end: 0 },
      alpha: { start: 0.9, end: 0 },
      frequency: 70,
      blendMode: 'ADD',
      tint: [0xff4433, 0xff8822, 0xffcc44],
      depth: 15
    });

    // Red heat pulse rising from horizon
    this.heat = this.setupEmitter('sparkle-pink', {
      y: { min: 430, max: 470 },
      x: { min: 0, max: 1024 },
      lifespan: 1500,
      speed: { min: -30, max: -10 },
      scale: { start: 0.5, end: 0 },
      alpha: { start: 0.5, end: 0 },
      frequency: 120,
      blendMode: 'ADD',
      tint: 0xff5522,
      depth: 2
    });

    this.cameras.main.shake(5000, 0.004);

    this.time.delayedCall(1000, () => {
      this.showDialogue('Narrator',
        'The dimension begins to crumble! The sky splits apart and reality fractures around River.',
        [{ text: 'Run!', callback: () => this.runSequence() }],
        'portrait-river'
      );
    });
  }
  runSequence() {
    this.cameras.main.shake(800, 0.02);
    this.showDialogue('Narrator',
      'River sprints through the collapsing world, dodging falling stars and breaking stone.',
      [{ text: 'Keep going!', callback: () => this.grabBook() }],
      'portrait-river'
    );
  }

  grabBook() {
    this.showDialogue('Narrator',
      'They reach the book. Its pages glow with an otherworldly light. River grabs it and... everything goes white.',
      [{ text: 'Continue', callback: () => this.goHome() }],
      'portrait-river'
    );
  }

  goHome() {
    audioManager.stopAllAmbient();
    this.cameras.main.fadeOut(1500, 255, 255, 255);
    this.cameras.main.once('camerafadeoutcomplete', () => {
      // Swap the crumbling realm for a warm, safe room
      if (this.sky) this.sky.setTexture('sky-home');
      if (this.ground) this.ground.setTexture('tile-house');
      if (this.cracks) this.cracks.setVisible(false);
      if (this.debris) this.debris.stop();
      if (this.heat) this.heat.stop();
      this.cameras.main.setBackgroundColor('#fbf3e1');
      this.cameras.main.fadeIn(2000);

      // Place home props (centred so their bases sit on the floor)
      this.add.image(180, GROUND_Y - 55, 'window').setScrollFactor(0).setDepth(7).setScale(0.9);
      this.add.image(760, GROUND_Y - 45, 'bed').setScrollFactor(0).setDepth(7).setScale(1.1);
      this.add.image(940, GROUND_Y - 60, 'shelf').setScrollFactor(0).setDepth(7).setScale(1.0);

      this.time.delayedCall(1500, () => {
        this.showDialogue('Narrator',
          'River wakes up at home. The book is in their hands. And Nadia is... smiling.',
          [{ text: 'Continue', callback: () => this.nadiaSpeaks() }],
          'portrait-river'
        );
      });
    });
  }

  nadiaSpeaks() {
    this.showDialogue('Nadia',
      'River... I feel different. I feel like I can understand things now. Like the words are... making sense.',
      [{ text: 'Continue', callback: () => this.riverSpeaks() }],
      'portrait-nadia'
    );
  }

  riverSpeaks() {
    this.showDialogue('River',
      'The Gift of Education... it worked.',
      [{ text: 'Continue', callback: () => this.finale() }],
      'portrait-river'
    );
  }

  finale() {
    const inv = this.gameState.inventory || [];
    const pieces = inv.length > 0 ? inv.length : 3;
    this.showDialogue('Narrator',
      `Nadia is cured. The family is whole again. River smiles, knowing the journey across ${pieces} worlds was worth every step.`,
      [{ text: 'The End', callback: () => this.credits() }],
      'portrait-river'
    );
  }

  credits() {
    this.cameras.main.fadeOut(3000, 0, 0, 0);
    this.cameras.main.once('camerafadeoutcomplete', () => {
      this.scene.start('IntroScene', { gameState: this.gameState });
    });
  }

  update() {
    this.handleMovement(250);
    this.updateParallax();
  }
}
