import { BaseScene } from './BaseScene.js';

export class HouseScene extends BaseScene {
  constructor() {
    super({ key: 'HouseScene', biomeId: 'house' });
  }

  create(data) {
    super.create(data);

    // Parallax: warm interior
    this.setupParallax([
      { key: 'player', speed: 0, y: 0, depth: -2, tint: 0x8B6914 },
      { key: 'player', speed: 0.1, y: 0, depth: -1, tint: 0xD2B48C }
    ]);

    // Background color for now
    this.cameras.main.setBackgroundColor('#2a1a0a');

    // Create player
    const player = this.createPlayer(200, 300);

    // Particles: dust motes
    this.setupParticleEmitter('particle', {
      y: { min: 0, max: 400 },
      lifespan: 6000,
      speed: { min: 2, max: 8 },
      scale: { start: 0.3, end: 0 },
      alpha: { start: 0.3, end: 0 },
      frequency: 800,
      blendMode: 'ADD',
      tint: 0xffeedd,
      depth: 1
    });

    // Intro sequence
    this.time.delayedCall(500, () => {
      this.showDialogue('Narrator', 'River sits at home, worried about their younger sibling Nadia.', [
        {
          text: 'Continue',
          callback: () => {
            this.showDialogue('Narrator', 'Nadia has fallen sick with a strange disease called "illiteracy".', [
              {
                text: 'Continue',
                callback: () => {
                  this.showDialogue('Grandparent', 'River, come here. I have something for you.', [
                    {
                      text: 'Continue',
                      callback: () => {
                        this.showDialogue('Grandparent', 'This is our Family Heirloom. A book that might be the key to solving Nadia\'s condition.', [
                          {
                            text: 'Continue',
                            callback: () => {
                              this.showDialogue('River', '...I\'ll try anything to help Nadia.', [
                                {
                                  text: 'Continue',
                                  callback: () => {
                                    this.time.delayedCall(500, () => {
                                      this.transitionToBiome('CandyLandScene');
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
          }
        }
      ]);
    });
  }

  update() {
    this.handleMovement(150);
    this.updateParallax();
  }

  transitionToBiome(sceneKey) {
    // Fade out transition
    this.cameras.main.fadeOut(1000, 0, 0, 0);
    this.cameras.main.once('camerafadeoutcomplete', () => {
      this.scene.start(sceneKey, { gameState: this.gameState });
    });
  }
}
