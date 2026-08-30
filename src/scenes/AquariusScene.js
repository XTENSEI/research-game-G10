import { BaseScene } from './BaseScene.js';

export class AquariusScene extends BaseScene {
  constructor() {
    super({ key: 'AquariusScene', biomeId: 'aquarius' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1000);
    this.cameras.main.setBackgroundColor('#0a2a4a');

    // Parallax: deep water, coral, fish layer
    this.setupParallax([
      { key: 'player', speed: 0.03, y: 0, depth: -3, tint: 0x0a1a3a },
      { key: 'player', speed: 0.1, y: 100, depth: -2, tint: 0x228B22 },
      { key: 'player', speed: 0.25, y: 250, depth: -1, tint: 0x4682B4 },
      { key: 'player', speed: 0.5, y: 380, depth: 0, tint: 0xf4a460 }
    ]);

    // Player
    const player = this.createPlayer(100, 380);

    // NPCs
    const gillbert = this.createNPC(350, 380, 'gillbert', 'Gillbert');
    const scalleon = this.createNPC(420, 380, 'scalleon', 'Scalleon');

    // Particles: bubbles rising
    this.setupParticleEmitter('bubble', {
      y: { min: 576, max: 576 },
      lifespan: 6000,
      speed: { min: 20, max: 50 },
      scale: { start: 0.3, end: 0.8 },
      alpha: { start: 0.5, end: 0.1 },
      frequency: 150,
      blendMode: 'ADD',
      tint: 0xaaddff,
      depth: 12
    });

    // Light rays
    this.setupParticleEmitter('particle', {
      x: { min: 200, max: 800 },
      y: -10,
      lifespan: 4000,
      speed: { min: 5, max: 15 },
      angle: { min: 80, max: 100 },
      scale: { start: 1.5, end: 0 },
      alpha: { start: 0.15, end: 0 },
      frequency: 500,
      blendMode: 'ADD',
      tint: 0x88ccff,
      depth: 2
    });

    // Talked tracking
    this.gillbertTalked = false;

    // Interaction
    this.physics.add.overlap(player, gillbert, () => {
      if (!this.gillbertTalked && !this.dialogueActive) {
        this.gillbertTalked = true;
        this.startGillbertDialogue();
      }
    });
  }

  startGillbertDialogue() {
    this.showDialogue('Gillbert', 'Hey, do you know where wild sea berries grow?', [
      {
        text: 'West?',
        callback: () => {
          this.showDialogue('Scalleon', 'Hmm, I believe it\'s in the west, I think?', [
            { text: 'Continue', callback: () => this.caveMazeIntro() }
          ]);
        }
      },
      {
        text: 'North?',
        callback: () => {
          this.showDialogue('Scalleon', 'No, real ones grow from the north. But the locals claim west.', [
            { text: 'Continue', callback: () => this.caveMazeIntro() }
          ]);
        }
      }
    ]);
  }

  caveMazeIntro() {
    this.showDialogue('Narrator', 'River reaches the Grave Cave Maze of Craze. A tablet with directional riddles guards the path.', [
      {
        text: 'Enter cave',
        callback: () => {
          // Placeholder: auto-complete cave for now
          this.showDialogue('Narrator', '[Cave Maze] Answer directional riddles to navigate. Wrong answers send you back.', [
            {
              text: 'Complete maze',
              callback: () => {
                this.showDialogue('Narrator', 'River finds an ingredient for the elixir in the depths of the cave.', [
                  {
                    text: 'Continue',
                    callback: () => this.transitionToNext()
                  }
                ]);
              }
            }
          ]);
        }
      }
    ]);
  }

  transitionToNext() {
    this.cameras.main.fadeOut(1000, 0, 0, 0);
    this.cameras.main.once('camerafadeoutcomplete', () => {
      this.scene.start('CosmaScene', { gameState: this.gameState });
    });
  }

  update() {
    this.handleMovement(160);
    this.updateParallax();
  }
}
