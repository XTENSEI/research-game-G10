import { BaseScene } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';

const JIN_QUESTIONS = [
  { q: 'Where did Jin say he was earlier?', opts: ['In the candy castle', 'In the red licorice forest', 'At the market', 'By the river'], correct: 1 },
  { q: 'What was Jin looking for in the forest?', opts: ['Gold coins', 'Licorice sticks', 'His horse', 'A way home'], correct: 1 },
  { q: 'What does Jin say everyone does in the forest?', opts: ['They give you riddles and questions', 'They sell candy', 'They fight you', 'They run away'], correct: 0 },
  { q: 'Why does Jin live alone?', opts: ['He likes quiet', 'He avoids riddles and questions', 'He has no friends', 'He is hiding'], correct: 1 },
  { q: "What does 'obfuscate' mean?", opts: ['To celebrate', 'To make vague and unintelligible', 'To break something', 'To run fast'], correct: 1 },
  { q: 'Which is the correct spelling?', opts: ['Acommodate', 'Accomodate', 'Accommodate', 'Acomadate'], correct: 2 },
  { q: "What did Jin say was a 'risk'?", opts: ['Asking River for help', 'Going to the castle', 'Eating licorice', 'Sleeping alone'], correct: 0 }
];

const JIN_REDUCED = [
  { q: 'Where was Jin earlier?', opts: ['The castle', 'The red licorice forest', 'The ocean', 'The mountain'], correct: 1 },
  { q: 'What was Jin looking for?', opts: ['Licorice sticks', 'His sword', 'A map', 'Food'], correct: 0 },
  { q: 'Why does Jin live alone?', opts: ['He is rich', 'He avoids riddles and questions', 'He is a king', 'He lost his family'], correct: 1 }
];

export class CandyLandScene extends BaseScene {
  constructor() {
    super({ key: 'CandyLandScene', biomeId: 'candy-land' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1500);
    this.cameras.main.setBackgroundColor('#2a0a1a');

    // Rich candy parallax
    this.setupParallax([
      { key: 'particle', speed: 0.02, y: 0, depth: -5, tint: 0xffb6c1, alpha: 0.3 },
      { key: 'particle', speed: 0.08, y: 80, depth: -4, tint: 0xffc0cb, alpha: 0.25 },
      { key: 'particle', speed: 0.15, y: 180, depth: -3, tint: 0xff69b4, alpha: 0.2 },
      { key: 'sparkle-pink', speed: 0.25, y: 300, depth: -2, tint: 0xffaacc, alpha: 0.4 },
      { key: 'particle', speed: 0.5, y: 420, depth: -1, tint: 0xff88aa, alpha: 0.3 }
    ]);

    const player = this.createPlayer(100, 380);
    const jin = this.createNPC(400, 380, 'jin', 'Jin', 'portrait-jin');

    // Cotton candy sparkles
    this.setupEmitter('sparkle-pink', {
      y: { min: -30, max: 200 },
      lifespan: 5000,
      speed: { min: 3, max: 12 },
      scale: { start: 0.5, end: 0 },
      alpha: { start: 0.7, end: 0 },
      frequency: 200,
      blendMode: 'ADD',
      tint: 0xff88cc,
      depth: 15
    });

    this.setupEmitter('sparkle-gold', {
      y: { min: 100, max: 300 },
      lifespan: 6000,
      speed: { min: 2, max: 8 },
      scale: { start: 0.4, end: 0 },
      alpha: { start: 0.5, end: 0 },
      frequency: 400,
      blendMode: 'ADD',
      tint: 0xffdd88,
      depth: 14
    });

    audioManager.playAmbientCandyLand();

    this.jinTalked = false;
    this.physics.add.overlap(player, jin, () => {
      if (!this.jinTalked && !this.dialogueActive) {
        this.jinTalked = true;
        this.startJinDialogue();
      }
    });
  }

  startJinDialogue() {
    this.showDialogue('Jin',
      'Howdy! I was just in the forest grabbing some licorice. Could you help me out here? I left my horse at home and whooo boy these sure are a bunch of sticks.',
      [
        { text: 'Where am I?', callback: () => this.jinWhere() },
        { text: 'No', callback: () => this.jinNo() }
      ],
      'portrait-jin'
    );
  }

  jinWhere() {
    this.showDialogue('Jin', "Well we're in the red licorice forest of course...", [
      { text: 'Continue', callback: () => this.jinWarn() }
    ], 'portrait-jin');
  }

  jinNo() {
    this.showDialogue('Jin', 'Oh, well where are you from? Aquarius? or Cosma?', [
      { text: 'Aquarius', callback: () => this.jinRegardless() },
      { text: 'Cosma', callback: () => this.jinRegardless() }
    ], 'portrait-jin');
  }

  jinRegardless() {
    this.showDialogue('Jin', 'Well regardless...', [
      { text: 'Continue', callback: () => this.jinWarn() }
    ], 'portrait-jin');
  }

  jinWarn() {
    this.showDialogue('Jin',
      "I gotta warn you about some of the wild life out here. They're pretty annoying to deal with.",
      [
        { text: 'In what way?', callback: () => this.jinAnnoying() },
        { text: 'Riddles?', callback: () => this.jinRiddles() }
      ],
      'portrait-jin'
    );
  }

  jinAnnoying() {
    this.showDialogue('Jin',
      "Everything is annoying. Everyone gives you riddles to solve, questions to answer, just to prove that they're better than you. But I don't bother with that stuff. That's why I live alone. Heck it was a risk even asking you for help.",
      [
        { text: 'Risk?', callback: () => this.jinDemonstrate() }
      ],
      'portrait-jin'
    );
  }

  jinRiddles() {
    this.showDialogue('Jin',
      "Yeah. You could ask me a riddle and beat me, that would be bad. You really aren't from here. Let me demonstrate.",
      [
        { text: 'Continue', callback: () => this.startBattle() }
      ],
      'portrait-jin'
    );
  }

  jinDemonstrate() {
    this.showDialogue('Jin',
      "Yeah. I mean you could beat me, that would be bad. You really aren't from here. Let me demonstrate.",
      [
        { text: 'Continue', callback: () => this.startBattle() }
      ],
      'portrait-jin'
    );
  }

  // ─── Tutorial Battle ────────────────────────────────────

  startBattle() {
    audioManager.playScreenFlash();
    this.cameras.main.flash(500, 255, 255, 255);

    this.time.delayedCall(600, () => {
      this.showDialogue('Jin',
        'Alright partner, time for a quiz! I will ask you questions about our conversation. Answer wisely!',
        [{ text: 'Bring it on', callback: () => this.runBattle() }],
        'portrait-jin'
      );
    });
  }

  runBattle() {
    this.battleCorrect = 0;
    this.battleWrong = 0;
    this.battleQuestions = this.shuffle([...JIN_QUESTIONS]);
    this.battleIndex = 0;
    this.askQuestion();
  }

  askQuestion() {
    if (this.battleIndex >= this.battleQuestions.length) {
      this.endBattle();
      return;
    }

    const q = this.battleQuestions[this.battleIndex];
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;

    // Battle overlay
    const overlay = this.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.75)
      .setDepth(80).setScrollFactor(0);

    // Question panel
    const panel = this.add.graphics().setDepth(81).setScrollFactor(0);
    panel.fillStyle(0x0d0d2a, 0.97);
    panel.fillRoundedRect(60, 40, w - 120, 130, 12);
    panel.lineStyle(2, 0xff69b4, 0.7);
    panel.strokeRoundedRect(60, 40, w - 120, 130, 12);

    // Progress
    const progress = this.add.text(w - 100, 55, `${this.battleIndex + 1}/${this.battleQuestions.length}`, {
      fontFamily: '"Courier New", monospace', fontSize: '14px', color: '#ff69b4'
    }).setOrigin(0.5).setDepth(82).setScrollFactor(0);

    // Question text
    const qText = this.add.text(w / 2, 95, q.q, {
      fontFamily: '"Courier New", monospace', fontSize: '17px', color: '#ffffff',
      wordWrap: { width: w - 200 }, align: 'center'
    }).setOrigin(0.5).setDepth(82).setScrollFactor(0);

    // Answer buttons
    const startY = 200;
    const btns = [];

    q.opts.forEach((opt, i) => {
      const by = startY + i * 52;
      const bg = this.add.graphics().setDepth(81).setScrollFactor(0);
      bg.fillStyle(0x1a1a3a, 0.95);
      bg.fillRoundedRect(w / 2 - 320, by, 640, 44, 8);
      bg.lineStyle(1, 0x4444aa, 0.5);
      bg.strokeRoundedRect(w / 2 - 320, by, 640, 44, 8);

      const label = this.add.text(w / 2, by + 22, `${String.fromCharCode(65 + i)}. ${opt}`, {
        fontFamily: '"Courier New", monospace', fontSize: '15px', color: '#999999'
      }).setOrigin(0.5).setDepth(82).setScrollFactor(0);

      const hitZone = this.add.zone(w / 2, by + 22, 640, 44)
        .setDepth(83).setScrollFactor(0).setInteractive({ useHandCursor: true });

      hitZone.on('pointerover', () => {
        bg.clear();
        bg.fillStyle(0x2a1a4a, 0.98);
        bg.fillRoundedRect(w / 2 - 320, by, 640, 44, 8);
        bg.lineStyle(2, 0xff69b4, 1);
        bg.strokeRoundedRect(w / 2 - 320, by, 640, 44, 8);
        label.setColor('#ff69b4');
      });

      hitZone.on('pointerout', () => {
        bg.clear();
        bg.fillStyle(0x1a1a3a, 0.95);
        bg.fillRoundedRect(w / 2 - 320, by, 640, 44, 8);
        bg.lineStyle(1, 0x4444aa, 0.5);
        bg.strokeRoundedRect(w / 2 - 320, by, 640, 44, 8);
        label.setColor('#999999');
      });

      hitZone.on('pointerdown', () => {
        this.answer(i, q.correct, overlay, panel, progress, qText, btns);
      });

      btns.push({ bg, label, hitZone });
    });

    this._battleElements = [overlay, panel, progress, qText, ...btns.flatMap(b => [b.bg, b.label, b.hitZone])];
  }

  answer(chosen, correct, overlay, panel, progress, qText, btns) {
    const isCorrect = chosen === correct;
    const w = this.cameras.main.width;

    if (isCorrect) {
      this.battleCorrect++;
      audioManager.playCorrect();
      this.cameras.main.flash(200, 0, 255, 100, false);
    } else {
      this.battleWrong++;
      audioManager.playWrong();
      this.cameras.main.flash(200, 255, 50, 50, false);
      this.cameras.main.shake(200, 0.012);
    }

    // Feedback text
    const fb = this.add.text(w / 2, 180, isCorrect ? 'CORRECT!' : 'WRONG!', {
      fontFamily: '"Courier New", monospace', fontSize: '28px', fontStyle: 'bold',
      color: isCorrect ? '#00ff66' : '#ff3333',
      stroke: '#000000', strokeThickness: 4
    }).setOrigin(0.5).setDepth(85).setScrollFactor(0);

    this.tweens.add({
      targets: fb, y: fb.y - 40, alpha: 0,
      duration: 1000, ease: 'Power2'
    });

    this.time.delayedCall(700, () => {
      // Clean up battle UI
      this._battleElements.forEach(el => el.destroy());
      fb.destroy();
      this.battleIndex++;
      this.askQuestion();
    });
  }

  endBattle() {
    const total = this.battleQuestions.length;
    const isPerfect = this.battleWrong === 0;

    if (isPerfect) {
      this.showDialogue('Jin',
        'Wow! You got every single one right! You are a natural partner!',
        [{ text: 'Thanks!', callback: () => this.jinReward() }],
        'portrait-jin'
      );
    } else if (this.battleCorrect > 0) {
      this.gameState.battleResult = 'with_losses';
      this.showDialogue('Jin',
        `You got ${total - this.battleWrong} out of ${total}! Not bad, but not perfect either. Thanks anyway partner! You better go along then, it's getting pretty late.`,
        [{ text: 'Continue', callback: () => this.goToCastle() }],
        'portrait-jin'
      );
    } else {
      this.showDialogue('Jin',
        "Aw shucks, looks like you don't got what it takes.",
        [
          { text: 'Challenge again', callback: () => this.retryBattle() },
          { text: 'Give up', callback: () => { this.jinTalked = false; } }
        ],
        'portrait-jin'
      );
    }
  }

  jinReward() {
    this.gameState.hasHorse = true;
    this.gameState.battleResult = 'perfect';
    this.showDialogue('Jin',
      "I really do appreciate it! Tell you what, you can take my horse. It's yours, really!",
      [{ text: 'Take the horse', callback: () => this.goToCastle() }],
      'portrait-jin'
    );
  }

  retryBattle() {
    this.battleCorrect = 0;
    this.battleWrong = 0;
    this.battleQuestions = this.shuffle([...JIN_REDUCED]);
    this.battleIndex = 0;
    audioManager.playScreenFlash();
    this.cameras.main.flash(300, 255, 255, 255);
    this.time.delayedCall(400, () => {
      this.showDialogue('Jin', 'Round 2! Fewer questions this time. Let us go!', [
        { text: 'Ready', callback: () => this.askQuestion() }
      ], 'portrait-jin');
    });
  }

  goToCastle() {
    this.showDialogue('Narrator',
      'River notices a big castle. They trek through Candy Land to reach it.',
      [{ text: 'Continue', callback: () => this.transitionTo('AquariusScene', 1500, 0x000033) }],
      'portrait-river'
    );
  }

  shuffle(arr) {
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
