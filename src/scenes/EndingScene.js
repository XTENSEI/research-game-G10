import { BaseScene } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';

export class EndingScene extends BaseScene {
  constructor() {
    super({ key: 'EndingScene', biomeId: 'house' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1000);
    this.cameras.main.setBackgroundColor('#1a0505');

    this.setupGround(0x2a0a0a, 460);

    this.setupParallax([
      { key: 'particle', speed: 0.02, y: 0, depth: -3, tint: 0x3a0a0a, alpha: 0.5 },
      { key: 'particle', speed: 0.1, y: 200, depth: -2, tint: 0x5a1a1a, alpha: 0.3 },
      { key: 'particle', speed: 0.3, y: 400, depth: -1, tint: 0x8a2a2a, alpha: 0.2 }
    ]);

    this.setupVignette(0.55, 89);

    const player = this.createPlayer(512, 300);

    // Crumbling particles (red/orange debris)
    this.setupEmitter('particle', {
      y: { min: 0, max: 576 },
      lifespan: 2000,
      speed: { min: 30, max: 80 },
      scale: { start: 0.6, end: 0 },
      alpha: { start: 0.8, end: 0 },
      frequency: 80,
      blendMode: 'ADD',
      tint: [0xff3333, 0xff6600, 0xffaa00],
      depth: 15
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
      this.cameras.main.setBackgroundColor('#f5f0e0');
      this.cameras.main.fadeIn(2000);

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
