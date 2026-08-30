import { BaseScene } from './BaseScene.js';

export class CandyLandScene extends BaseScene {
  constructor() {
    super({ key: 'CandyLandScene', biomeId: 'candy-land' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1000);
    this.cameras.main.setBackgroundColor('#ffb6c1');

    // Parallax layers: pink sky, cotton candy clouds, lemon ground, lollipop bushes
    this.setupParallax([
      { key: 'player', speed: 0.05, y: 0, depth: -3, tint: 0xffb6c1 },
      { key: 'player', speed: 0.15, y: 100, depth: -2, tint: 0xffc0cb },
      { key: 'player', speed: 0.3, y: 200, depth: -1, tint: 0xffdd44 },
      { key: 'player', speed: 0.6, y: 350, depth: 0, tint: 0xff69b4 }
    ]);

    // Player
    const player = this.createPlayer(100, 350);

    // Jin NPC (gingerbread man)
    const jin = this.createNPC(400, 350, 'jin', 'Jin');

    // Particles: sparkles + cotton candy float
    this.setupParticleEmitter('particle', {
      y: { min: -50, max: 200 },
      lifespan: 5000,
      speed: { min: 5, max: 15 },
      scale: { start: 0.4, end: 0 },
      alpha: { start: 0.7, end: 0 },
      frequency: 300,
      blendMode: 'ADD',
      tint: 0xff69b4,
      depth: 15
    });

    this.setupParticleEmitter('particle', {
      y: { min: 0, max: 100 },
      lifespan: 8000,
      speed: { min: 3, max: 8 },
      scale: { start: 0.6, end: 0.1 },
      alpha: { start: 0.5, end: 0 },
      frequency: 600,
      blendMode: 'ADD',
      tint: 0xffc0cb,
      depth: 14
    });

    // Jin intro dialogue trigger
    this.jinTalked = false;

    // Interaction zone
    this.physics.add.overlap(player, jin, () => {
      if (!this.jinTalked && !this.dialogueActive) {
        this.jinTalked = true;
        this.startJinDialogue();
      }
    });
  }

  startJinDialogue() {
    this.showDialogue('Jin', 'Howdy! I was just in the forest grabbing some licorice. Could you help me out here? I left my horse at home and whooo boy these sure are a bunch of sticks.', [
      {
        text: 'Where am I?',
        callback: () => {
          this.showDialogue('Jin', 'Well we\'re in the red licorice forest of course...', [
            { text: 'Continue', callback: () => this.jinDialogue2() }
          ]);
        }
      },
      {
        text: 'No',
        callback: () => {
          this.showDialogue('Jin', 'Oh, well where are you from? Aquarius? or Cosma?', [
            {
              text: 'Aquarius',
              callback: () => {
                this.showDialogue('Jin', 'Well regardless...', [
                  { text: 'Continue', callback: () => this.jinDialogue2() }
                ]);
              }
            },
            {
              text: 'Cosma',
              callback: () => {
                this.showDialogue('Jin', 'Well regardless...', [
                  { text: 'Continue', callback: () => this.jinDialogue2() }
                ]);
              }
            }
          ]);
        }
      }
    ]);
  }

  jinDialogue2() {
    this.showDialogue('Jin', 'I gotta warn you about some of the wild life out here. They\'re pretty annoying to deal with.', [
      {
        text: 'In what way?',
        callback: () => {
          this.showDialogue('Jin', 'Everything is annoying. Everyone gives you riddles to solve, questions to answer, just to prove that they\'re better than you.', [
            { text: 'Continue', callback: () => this.jinDialogue3() }
          ]);
        }
      },
      {
        text: 'Riddles?',
        callback: () => {
          this.showDialogue('Jin', 'Yeah. You could ask me a riddle and beat me, that would be bad. You really aren\'t from here. Let me demonstrate.', [
            { text: 'Continue', callback: () => this.startTutorialBattle() }
          ]);
        }
      }
    ]);
  }

  jinDialogue3() {
    this.showDialogue('Jin', 'But I don\'t bother with that stuff. That\'s why I live alone. Heck it was a risk even asking you for help.', [
      {
        text: 'Risk?',
        callback: () => {
          this.showDialogue('Jin', 'Yeah. I mean you could beat me, that would be bad. You really aren\'t from here. Let me demonstrate.', [
            { text: 'Continue', callback: () => this.startTutorialBattle() }
          ]);
        }
      }
    ]);
  }

  startTutorialBattle() {
    // Flash effect for battle transition
    this.cameras.main.flash(500, 255, 255, 255);
    this.time.delayedCall(500, () => {
      this.showDialogue('Jin', '[Tutorial Battle] Jin asks you questions about your conversation. Answer correctly!', [
        {
          text: 'Ready',
          callback: () => {
            // Placeholder: auto-win for now
            this.showDialogue('Jin', 'Wow! You are a natural! Can you help me pick up these licorice sticks now...', [
              {
                text: 'Continue',
                callback: () => {
                  this.showDialogue('Narrator', 'River helps Jin with his sticks.', [
                    {
                      text: 'Continue',
                      callback: () => {
                        this.gameState.hasHorse = true;
                        this.showDialogue('Jin', 'I really do appreciate it, partner! If you want, you could go ahead and take my horse. It\'s yours, really!', [
                          {
                            text: 'Take horse',
                            callback: () => this.goToCastle()
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

  goToCastle() {
    this.showDialogue('Narrator', 'River notices a big castle. They trek through Candy Land to reach it.', [
      {
        text: 'Continue',
        callback: () => {
          this.cameras.main.fadeOut(1000, 0, 0, 0);
          this.cameras.main.once('camerafadeoutcomplete', () => {
            // For now, transition to Aquarius (castle boss + transition to next biome)
            this.scene.start('AquariusScene', { gameState: this.gameState });
          });
        }
      }
    ]);
  }

  update() {
    this.handleMovement(200);
    this.updateParallax();
  }
}
