import { BaseScene } from './BaseScene.js';

const JIN_QUESTIONS = {
  full_set: [
    {
      q: 'Where did Jin say he was earlier?',
      opts: ['In the candy castle', 'In the red licorice forest', 'At the market', 'By the river'],
      correct: 1
    },
    {
      q: 'What was Jin looking for in the forest?',
      opts: ['Gold coins', 'Licorice sticks', 'His horse', 'A way home'],
      correct: 1
    },
    {
      q: 'What does Jin say everyone does in the forest?',
      opts: ['They give you riddles and questions', 'They sell candy', 'They fight you', 'They run away'],
      correct: 0
    },
    {
      q: 'Why does Jin live alone?',
      opts: ['He likes quiet', 'He avoids riddles and questions', 'He has no friends', 'He is hiding'],
      correct: 1
    },
    {
      q: "What does 'obfuscate' mean?",
      opts: ['To celebrate', 'To make vague and unintelligible', 'To break something', 'To run fast'],
      correct: 1
    },
    {
      q: 'Which is the correct spelling?',
      opts: ['Acommodate', 'Accomodate', 'Accommodate', 'Acomadate'],
      correct: 2
    },
    {
      q: "What did Jin say was a 'risk'?",
      opts: ['Asking River for help', 'Going to the castle', 'Eating licorice', 'Sleeping alone'],
      correct: 0
    }
  ],
  reduced_set: [
    {
      q: 'Where was Jin earlier?',
      opts: ['The castle', 'The red licorice forest', 'The ocean', 'The mountain'],
      correct: 1
    },
    {
      q: 'What was Jin looking for?',
      opts: ['Licorice sticks', 'His sword', 'A map', 'Food'],
      correct: 0
    },
    {
      q: 'Why does Jin live alone?',
      opts: ['He is rich', 'He avoids riddles and questions', 'He is a king', 'He lost his family'],
      correct: 1
    }
  ]
};

export class CandyLandScene extends BaseScene {
  constructor() {
    super({ key: 'CandyLandScene', biomeId: 'candy-land' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1000);
    this.cameras.main.setBackgroundColor('#ffb6c1');

    this.setupParallax([
      { key: 'player', speed: 0.05, y: 0, depth: -3, tint: 0xffb6c1 },
      { key: 'player', speed: 0.15, y: 100, depth: -2, tint: 0xffc0cb },
      { key: 'player', speed: 0.3, y: 200, depth: -1, tint: 0xffdd44 },
      { key: 'player', speed: 0.6, y: 350, depth: 0, tint: 0xff69b4 }
    ]);

    const player = this.createPlayer(100, 350);
    const jin = this.createNPC(400, 350, 'jin', 'Jin');

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

    this.jinTalked = false;
    this.battleCorrect = 0;
    this.battleWrong = 0;
    this.battleQuestions = [];
    this.battleIndex = 0;

    this.physics.add.overlap(player, jin, () => {
      if (!this.jinTalked && !this.dialogueActive) {
        this.jinTalked = true;
        this.startJinDialogue();
      }
    });
  }

  // ─── Jin conversation dialogue ──────────────────────────

  startJinDialogue() {
    this.showDialogue('Jin', 'Howdy! I was just in the forest grabbing some licorice. Could you help me out here? I left my horse at home and whooo boy these sure are a bunch of sticks.', [
      {
        text: 'Where am I?',
        callback: () => {
          this.showDialogue('Jin', "Well we're in the red licorice forest of course...", [
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
    this.showDialogue('Jin', "I gotta warn you about some of the wild life out here. They're pretty annoying to deal with.", [
      {
        text: 'In what way?',
        callback: () => {
          this.showDialogue('Jin', "Everything is annoying. Everyone gives you riddles to solve, questions to answer, just to prove that they're better than you.", [
            { text: 'Continue', callback: () => this.jinDialogue3() }
          ]);
        }
      },
      {
        text: 'Riddles?',
        callback: () => {
          this.showDialogue('Jin', "Yeah. You could ask me a riddle and beat me, that would be bad. You really aren't from here. Let me demonstrate.", [
            { text: 'Continue', callback: () => this.startTutorialBattle() }
          ]);
        }
      }
    ]);
  }

  jinDialogue3() {
    this.showDialogue('Jin', "But I don't bother with that stuff. That's why I live alone. Heck it was a risk even asking you for help.", [
      {
        text: 'Risk?',
        callback: () => {
          this.showDialogue('Jin', "Yeah. I mean you could beat me, that would be bad. You really aren't from here. Let me demonstrate.", [
            { text: 'Continue', callback: () => this.startTutorialBattle() }
          ]);
        }
      }
    ]);
  }

  // ─── Tutorial battle ────────────────────────────────────

  startTutorialBattle() {
    this.cameras.main.flash(500, 255, 255, 255);
    this.time.delayedCall(500, () => {
      this.showDialogue('Jin', 'Alright partner, time for a quiz! I will ask you questions about our conversation. Answer wisely!', [
        {
          text: 'Bring it on',
          callback: () => {
            this.battleCorrect = 0;
            this.battleWrong = 0;
            this.battleQuestions = this.shuffleArray([...JIN_QUESTIONS.full_set]);
            this.battleIndex = 0;
            this.askBattleQuestion();
          }
        }
      ]);
    });
  }

  askBattleQuestion() {
    if (this.battleIndex >= this.battleQuestions.length) {
      this.endBattle();
      return;
    }

    const q = this.battleQuestions[this.battleIndex];
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;

    // Battle UI
    const overlay = this.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.7).setDepth(20).setScrollFactor(0);

    // Question box
    const qBox = this.add.graphics().setDepth(21).setScrollFactor(0);
    qBox.fillStyle(0x1a1a2e, 0.95);
    qBox.fillRoundedRect(w / 2 - 350, 30, 700, 120, 12);
    qBox.lineStyle(2, 0xff69b4, 1);
    qBox.strokeRoundedRect(w / 2 - 350, 30, 700, 120, 12);

    // Score display
    const scoreText = this.add.text(w / 2, 50, `Question ${this.battleIndex + 1}/${this.battleQuestions.length}`, {
      fontFamily: 'Arial', fontSize: '14px', color: '#ff69b4'
    }).setOrigin(0.5).setDepth(22).setScrollFactor(0);

    // Question text
    const qText = this.add.text(w / 2, 90, q.q, {
      fontFamily: 'Arial', fontSize: '18px', color: '#ffffff',
      wordWrap: { width: 660 }, align: 'center'
    }).setOrigin(0.5).setDepth(22).setScrollFactor(0);

    // Answer buttons
    const btns = [];
    const startY = 180;
    const btnSpacing = 55;

    q.opts.forEach((opt, i) => {
      const btnY = startY + i * btnSpacing;
      const btn = this.add.graphics().setDepth(21).setScrollFactor(0);
      btn.fillStyle(0x2a2a4a, 0.9);
      btn.fillRoundedRect(w / 2 - 300, btnY, 600, 44, 8);
      btn.lineStyle(1, 0x6666aa, 1);
      btn.strokeRoundedRect(w / 2 - 300, btnY, 600, 44, 8);

      const label = this.add.text(w / 2, btnY + 22, `${String.fromCharCode(65 + i)}. ${opt}`, {
        fontFamily: 'Arial', fontSize: '16px', color: '#cccccc'
      }).setOrigin(0.5).setDepth(22).setScrollFactor(0);

      // Hit zone
      const hitZone = this.add.zone(w / 2, btnY + 22, 600, 44)
        .setDepth(23).setScrollFactor(0).setInteractive({ useHandCursor: true });

      hitZone.on('pointerover', () => {
        btn.clear();
        btn.fillStyle(0x4a3a6a, 0.95);
        btn.fillRoundedRect(w / 2 - 300, btnY, 600, 44, 8);
        btn.lineStyle(2, 0xff69b4, 1);
        btn.strokeRoundedRect(w / 2 - 300, btnY, 600, 44, 8);
        label.setColor('#ff69b4');
      });

      hitZone.on('pointerout', () => {
        btn.clear();
        btn.fillStyle(0x2a2a4a, 0.9);
        btn.fillRoundedRect(w / 2 - 300, btnY, 600, 44, 8);
        btn.lineStyle(1, 0x6666aa, 1);
        btn.strokeRoundedRect(w / 2 - 300, btnY, 600, 44, 8);
        label.setColor('#cccccc');
      });

      hitZone.on('pointerdown', () => {
        this.answerBattleQuestion(i, q.correct, overlay, qBox, scoreText, qText, btns);
      });

      btns.push({ btn, label, hitZone, btnY });
    });
  }

  answerBattleQuestion(chosen, correct, overlay, qBox, scoreText, qText, btns) {
    const isCorrect = chosen === correct;
    const w = this.cameras.main.width;

    if (isCorrect) {
      this.battleCorrect++;
      // Green flash
      this.cameras.main.flash(200, 0, 255, 100, false);
    } else {
      this.battleWrong++;
      // Red flash + shake
      this.cameras.main.flash(200, 255, 50, 50, false);
      this.cameras.main.shake(200, 0.01);
    }

    // Show correct/wrong indicator
    const indicator = this.add.text(w / 2, 160, isCorrect ? 'CORRECT!' : 'WRONG!', {
      fontFamily: 'Arial', fontSize: '24px', fontStyle: 'bold',
      color: isCorrect ? '#00ff66' : '#ff3333'
    }).setOrigin(0.5).setDepth(25).setScrollFactor(0);

    this.tweens.add({
      targets: indicator,
      y: indicator.y - 30,
      alpha: 0,
      duration: 1200,
      onComplete: () => indicator.destroy()
    });

    // Clean up UI elements
    this.time.delayedCall(800, () => {
      overlay.destroy();
      qBox.destroy();
      scoreText.destroy();
      qText.destroy();
      btns.forEach(b => { b.btn.destroy(); b.label.destroy(); b.hitZone.destroy(); });

      this.battleIndex++;
      this.askBattleQuestion();
    });
  }

  endBattle() {
    const total = this.battleQuestions.length;
    const isPerfectClear = this.battleWrong === 0;

    if (isPerfectClear) {
      this.showDialogue('Jin', 'Wow! You got every single one right! You are a natural partner!', [
        {
          text: 'Thanks!',
          callback: () => {
            this.showDialogue('Jin', "I really do appreciate it! Tell you what, you can take my horse. It's yours, really!", [
              {
                text: 'Take the horse',
                callback: () => {
                  this.gameState.hasHorse = true;
                  this.gameState.battleResult = 'perfect';
                  this.goToCastle();
                }
              }
            ]);
          }
        }
      ]);
    } else if (this.battleWrong < total) {
      // Won but with mistakes
      this.showDialogue('Jin', `You got ${total - this.battleWrong} out of ${total}! Not bad, but not perfect either.`, [
        {
          text: 'Continue',
          callback: () => {
            this.gameState.hasHorse = false;
            this.gameState.battleResult = 'with_losses';
            this.showDialogue('Jin', "Thanks anyway partner! You better go along then, it's getting pretty late.", [
              { text: 'Continue', callback: () => this.goToCastle() }
            ]);
          }
        }
      ]);
    } else {
      // Lost all
      this.showDialogue('Jin', "Aw shucks, looks like you don't got what it takes.", [
        {
          text: 'Challenge again',
          callback: () => {
            // Retry with fewer questions
            this.battleCorrect = 0;
            this.battleWrong = 0;
            this.battleQuestions = this.shuffleArray([...JIN_QUESTIONS.reduced_set]);
            this.battleIndex = 0;
            this.cameras.main.flash(300, 255, 255, 255);
            this.time.delayedCall(300, () => {
              this.showDialogue('Jin', 'Round 2! Fewer questions this time. Let us go!', [
                { text: 'Ready', callback: () => this.askBattleQuestion() }
              ]);
            });
          }
        },
        {
          text: 'Give up',
          callback: () => {
            this.gameState.hasHorse = false;
            this.gameState.battleResult = 'gave_up';
            this.jinTalked = false;
            this.showDialogue('Jin', "Well alright then. Maybe next time partner.", []);
          }
        }
      ]);
    }
  }

  goToCastle() {
    this.showDialogue('Narrator', 'River notices a big castle while leaving Jin\'s house. They trek through Candy Land to reach it.', [
      {
        text: 'Continue',
        callback: () => {
          this.cameras.main.fadeOut(1000, 0, 0, 0);
          this.cameras.main.once('camerafadeoutcomplete', () => {
            this.scene.start('AquariusScene', { gameState: this.gameState });
          });
        }
      }
    ]);
  }

  shuffleArray(arr) {
    for (let i = arr.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [arr[i], arr[j]] = [arr[j], arr[i]];
    }
    return arr;
  }

  update() {
    this.handleMovement(200);
    this.updateParallax();
  }
}
