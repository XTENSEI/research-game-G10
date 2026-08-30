import { BaseScene } from './BaseScene.js';

export class EndingScene extends BaseScene {
  constructor() {
    super({ key: 'EndingScene', biomeId: 'house' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(2000);
    this.cameras.main.setBackgroundColor('#1a0a0a');

    // Player
    const player = this.createPlayer(512, 300);

    // Particles: crumbling dimension
    this.setupParticleEmitter('particle', {
      y: { min: 0, max: 576 },
      lifespan: 2000,
      speed: { min: 30, max: 80 },
      scale: { start: 0.8, end: 0 },
      alpha: { start: 0.8, end: 0 },
      frequency: 100,
      blendMode: 'ADD',
      tint: 0xff3333,
      depth: 15
    });

    // Screen shake
    this.cameras.main.shake(5000, 0.005);

    this.time.delayedCall(1000, () => {
      this.showDialogue('Narrator', 'The dimension begins to crumble! The sky splits apart and reality fractures.', [
        {
          text: 'Run!',
          callback: () => {
            this.showDialogue('Narrator', 'River sprints through the collapsing world, dodging falling stars and breaking stone.', [
              {
                text: 'Continue',
                callback: () => {
                  this.showDialogue('Narrator', 'They reach the book. Its pages glow. River grabs it and... everything goes white.', [
                    {
                      text: 'Continue',
                      callback: () => this.showEnding()
                    }
                  ]);
                }
              }
            ]);
          }
        }
      ]);
    });
  }

  showEnding() {
    this.cameras.main.fadeOut(1000, 255, 255, 255);
    this.cameras.main.once('camerafadeoutcomplete', () => {
      this.cameras.main.setBackgroundColor('#fff8dc');
      this.cameras.main.fadeIn(1000);

      this.showDialogue('Narrator', 'River wakes up at home. The book is in their hands. Nadia is smiling.', [
        {
          text: 'Continue',
          callback: () => {
            this.showDialogue('Nadia', 'River... I feel different. I feel like I can understand things now.', [
              {
                text: 'Continue',
                callback: () => {
                  this.showDialogue('River', 'The Gift of Education... it worked.', [
                    {
                      text: 'Continue',
                      callback: () => {
                        this.showDialogue('Narrator', 'Nadia is cured. The family is whole again. River smiles, knowing the journey was worth it.', [
                          {
                            text: 'The End',
                            callback: () => {
                              this.cameras.main.fadeOut(2000);
                              this.cameras.main.once('camerafadeoutcomplete', () => {
                                // Return to title (placeholder)
                                this.scene.start('HouseScene', { gameState: this.gameState });
                              });
                            }
                          }
                        ]);
                      }
                    }
                  ]);
                }
              }
            ]);
          }
        }
      ]);
    });
  }

  update() {
    this.handleMovement(250);
    this.updateParallax();
  }
}
