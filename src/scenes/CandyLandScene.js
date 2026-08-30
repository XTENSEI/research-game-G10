import { BaseScene } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';

const JIN_QUESTIONS = [
  { q: 'Where did Jin say he was earlier?', opts: ['In the candy castle', 'In the red licorice forest', 'At the market', 'By the river'], correct: 1 },
  { q: 'What was Jin looking for?', opts: ['Gold coins', 'Licorice sticks', 'His horse', 'A way home'], correct: 1 },
  { q: 'What does everyone do in the forest?', opts: ['Give you riddles and questions', 'Sell candy', 'Fight you', 'Run away'], correct: 0 },
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

    this.cameras.main.fadeIn(2000);
    this.cameras.main.setBackgroundColor('#2a0a1a');

    this.setupParallax([
      { key: 'particle', speed: 0.02, y: 0, depth: -5, tint: 0xffb6c1, alpha: 0.3 },
      { key: 'particle', speed: 0.08, y: 80, depth: -4, tint: 0xffc0cb, alpha: 0.25 },
      { key: 'particle', speed: 0.15, y: 180, depth: -3, tint: 0xff69b4, alpha: 0.2 },
      { key: 'sparkle-pink', speed: 0.25, y: 300, depth: -2, tint: 0xffaacc, alpha: 0.4 },
      { key: 'particle', speed: 0.5, y: 420, depth: -1, tint: 0xff88aa, alpha: 0.3 }
    ]);

    this.createPlayer(100, 380);

    // Cotton candy sparkles
    this.setupEmitter('sparkle-pink', {
      y: { min: -30, max: 200 }, lifespan: 5000, speed: { min: 3, max: 12 },
      scale: { start: 0.5, end: 0 }, alpha: { start: 0.7, end: 0 },
      frequency: 200, blendMode: 'ADD', tint: 0xff88cc, depth: 15
    });

    audioManager.playAmbientCandyLand();

    // Script beat: River sits in Candy Land confused
    this.time.delayedCall(1000, () => {
      this.showDialogue('River', 'Where... where am I? This looks nothing like home.', [
        { text: 'Look around', callback: () => this.lookAround() }
      ], 'portrait-river');
    });
  }

  lookAround() {
    // Script beat: They look around their surroundings and notice a castle
    this.showDialogue('Narrator', 'River looks around. Pink sky, cotton candy clouds, lollipop bushes everywhere. In the distance, a massive castle looms.', [
      { text: 'Keep walking', callback: () => this.walkForest() }
    ], 'portrait-river');
  }

  walkForest() {
    // Script beat: River starts walking on the forest path when they encounter a gingerbread man
    const jin = this.createNPC(400, 380, 'jin', 'Jin', 'portrait-jin');

    this.showDialogue('Narrator', 'River walks along the red licorice forest path. Something small and brown steps out from behind a lollipop bush.', [
      { text: 'Continue', callback: () => this.jinIntro() }
    ]);
  }

  jinIntro() {
    // Script: Jin says "Howdy!" about licorice, asks if from around here
    this.showDialogue('Jin',
      'Howdy! I was just in the forest earlier grabbing some licorice, could you help me out here? I left my horse at home and whooo boy these sure are a bunch of sticks.',
      [
        { text: 'Say, you from around here?', callback: () => this.jinWhere() }
      ], 'portrait-jin'
    );
  }

  jinWhere() {
    // Script: Where am I / Where are we / No
    this.showDialogue('Jin', "Say, you from around here?", [
      { text: 'Where am I?', callback: () => this.jinForest() },
      { text: 'Where are we?', callback: () => this.jinForest() },
      { text: 'No', callback: () => this.jinNo() }
    ], 'portrait-jin');
  }

  jinForest() {
    // Script: "Well we're in the red licorice forest of course..."
    this.showDialogue('Jin', "Well we're in the red licorice forest of course...", [
      { text: 'Continue', callback: () => this.jinWarn() }
    ], 'portrait-jin');
  }

  jinNo() {
    // Script: "Oh, well where are you from? Aquarius? or Cosma?" + options including their actual home
    this.showDialogue('Jin', 'Oh, well where are you from? Aquarius? or Cosma?', [
      { text: 'Aquarius', callback: () => this.jinRegardless() },
      { text: 'Cosma', callback: () => this.jinRegardless() },
      { text: '*the place where they are actually from', callback: () => this.jinRegardless() }
    ], 'portrait-jin');
  }

  jinRegardless() {
    this.showDialogue('Jin', 'Well regardless...', [
      { text: 'Continue', callback: () => this.jinWarn() }
    ], 'portrait-jin');
  }

  jinWarn() {
    // Script: "I gotta warn you about some of the wild life out here.."
    this.showDialogue('Jin', "I gotta warn you about some of the wild life out here. They're pretty annoying to deal with.", [
      { text: 'In what way?', callback: () => this.jinAnnoying() },
      { text: 'How?', callback: () => this.jinAnnoying() },
      { text: 'Annoying?', callback: () => this.jinAnnoying() }
    ], 'portrait-jin');
  }

  jinAnnoying() {
    // Script: "Everything is annoying" ... "Everyone gives you riddles..."
    this.showDialogue('Jin',
      "Well, yeah? Everything is annoying. Everyone gives you riddles to solve, questions to answer, just to prove that they're better than you. And there are a lot of them here in the forest. But I don't bother with that stuff, that's why I live alone, heck it was a risk even asking you for help.",
      [
        { text: 'Risk?', callback: () => this.jinRisk() },
        { text: 'Riddles?', callback: () => this.jinRiddles() },
        { text: 'Questions?', callback: () => this.jinRiddles() }
      ], 'portrait-jin'
    );
  }

  jinRisk() {
    // Script: "Yeah it's a risk? I mean you could beat me, that would be bad"
    this.showDialogue('Jin',
      "Yeah it's a risk? I mean you could beat me, that would be bad. You really aren't from here. Like at all. Let me demonstrate.",
      [{ text: 'Continue', callback: () => this.startBattle() }],
      'portrait-jin'
    );
  }

  jinRiddles() {
    // Script: "Yeah? I mean you could ask me a riddle/question and beat me"
    this.showDialogue('Jin',
      "Yeah? I mean you could ask me a riddle/question and beat me, that would be bad. You really aren't from here. Like at all. Let me demonstrate.",
      [{ text: 'Continue', callback: () => this.startBattle() }],
      'portrait-jin'
    );
  }

  // ─── TUTORIAL BATTLE ────────────────────────────────────

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
    if (this.battleIndex >= this.battleQuestions.length) { this.endBattle(); return; }
    const q = this.battleQuestions[this.battleIndex];
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;

    const overlay = this.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.75).setDepth(80).setScrollFactor(0);
    const panel = this.add.graphics().setDepth(81).setScrollFactor(0);
    panel.fillStyle(0x0d0d2a, 0.97);
    panel.fillRoundedRect(60, 40, w - 120, 130, 12);
    panel.lineStyle(2, 0xff69b4, 0.7);
    panel.strokeRoundedRect(60, 40, w - 120, 130, 12);

    const progress = this.add.text(w - 100, 55, `${this.battleIndex + 1}/${this.battleQuestions.length}`, {
      fontFamily: '"Courier New", monospace', fontSize: '14px', color: '#ff69b4'
    }).setOrigin(0.5).setDepth(82).setScrollFactor(0);

    const qText = this.add.text(w / 2, 95, q.q, {
      fontFamily: '"Courier New", monospace', fontSize: '17px', color: '#ffffff',
      wordWrap: { width: w - 200 }, align: 'center'
    }).setOrigin(0.5).setDepth(82).setScrollFactor(0);

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

      const hitZone = this.add.zone(w / 2, by + 22, 640, 44).setDepth(83).setScrollFactor(0).setInteractive({ useHandCursor: true });
      hitZone.on('pointerover', () => { bg.clear(); bg.fillStyle(0x2a1a4a, 0.98); bg.fillRoundedRect(w / 2 - 320, by, 640, 44, 8); bg.lineStyle(2, 0xff69b4, 1); bg.strokeRoundedRect(w / 2 - 320, by, 640, 44, 8); label.setColor('#ff69b4'); });
      hitZone.on('pointerout', () => { bg.clear(); bg.fillStyle(0x1a1a3a, 0.95); bg.fillRoundedRect(w / 2 - 320, by, 640, 44, 8); bg.lineStyle(1, 0x4444aa, 0.5); bg.strokeRoundedRect(w / 2 - 320, by, 640, 44, 8); label.setColor('#999999'); });
      hitZone.on('pointerdown', () => this.answer(i, q.correct, [overlay, panel, progress, qText, ...btns.flatMap(b => [b.bg, b.label, b.hitZone])]));
      btns.push({ bg, label, hitZone });
    });
  }

  answer(chosen, correct, elements) {
    const isCorrect = chosen === correct;
    const w = this.cameras.main.width;
    if (isCorrect) { this.battleCorrect++; audioManager.playCorrect(); this.cameras.main.flash(200, 0, 255, 100, false); }
    else { this.battleWrong++; audioManager.playWrong(); this.cameras.main.flash(200, 255, 50, 50, false); this.cameras.main.shake(200, 0.012); }

    const fb = this.add.text(w / 2, 180, isCorrect ? 'CORRECT!' : 'WRONG!', {
      fontFamily: '"Courier New", monospace', fontSize: '28px', fontStyle: 'bold',
      color: isCorrect ? '#00ff66' : '#ff3333', stroke: '#000000', strokeThickness: 4
    }).setOrigin(0.5).setDepth(85).setScrollFactor(0);
    this.tweens.add({ targets: fb, y: fb.y - 40, alpha: 0, duration: 1000 });

    this.time.delayedCall(700, () => { elements.forEach(el => el.destroy()); fb.destroy(); this.battleIndex++; this.askQuestion(); });
  }

  endBattle() {
    const total = this.battleQuestions.length;
    if (this.battleWrong === 0) {
      // Script: perfect clear - "Wow! You are a natural!"
      this.gameState.hasHorse = true;
      this.gameState.battleResult = 'perfect';
      this.showDialogue('Jin', 'Wow! You are a natural! Can you help me pick up these licorice sticks now...', [
        { text: 'Help Jin', callback: () => this.helpJin() }
      ], 'portrait-jin');
    } else if (this.battleCorrect > 0) {
      this.gameState.battleResult = 'with_losses';
      this.showDialogue('Jin', "Aw shucks, looks like you don't got what it takes. But you got some right at least.", [
        { text: 'Continue', callback: () => this.helpJin() }
      ], 'portrait-jin');
    } else {
      // Script: lose - challenge again or give up
      this.showDialogue('Jin', "Aw shucks, looks like you don't got what it takes.", [
        { text: 'Challenge him again', callback: () => this.retryBattle() },
        { text: 'Give up', callback: () => { this.jinTalked = false; } }
      ], 'portrait-jin');
    }
  }

  retryBattle() {
    this.battleCorrect = 0;
    this.battleWrong = 0;
    this.battleQuestions = this.shuffle([...JIN_REDUCED]);
    this.battleIndex = 0;
    audioManager.playScreenFlash();
    this.cameras.main.flash(300, 255, 255, 255);
    this.time.delayedCall(400, () => {
      this.showDialogue('Jin', 'Round 2! Fewer questions this time.', [
        { text: 'Ready', callback: () => this.askQuestion() }
      ], 'portrait-jin');
    });
  }

  helpJin() {
    // Script: River helps Jin with his sticks
    this.showDialogue('Narrator', 'River helps Jin pick up the licorice sticks.', [
      { text: 'Continue', callback: () => this.jinHorseOrFoot() }
    ]);
  }

  jinHorseOrFoot() {
    if (this.gameState.hasHorse) {
      // Script: "I really do appreciate it, partner! If you want, you could go ahead and take my horse."
      this.showDialogue('Jin', "I really do appreciate it, partner! If you want, you could go ahead and take my horse. It's yours, really!", [
        { text: 'Take the horse', callback: () => this.findBook() }
      ], 'portrait-jin');
    } else {
      // Script: "Thanks partner! You better go along then, it's getting pretty late"
      this.showDialogue('Jin', "Thanks partner! You better go along then, it's getting pretty late.", [
        { text: 'Continue', callback: () => this.findBook() }
      ], 'portrait-jin');
    }
  }

  findBook() {
    // Script: River notices a big castle while at Jin's house, notices the book on the table
    this.showDialogue('Narrator', 'While at Jin\'s house, River notices a big castle in the distance. Then they spot something on the table. Their Family Heirloom. The book.', [
      { text: 'Continue', callback: () => this.bookDialogue() }
    ]);
  }

  bookDialogue() {
    // Script: dialogue options about the book
    this.showDialogue('Jin', "Oh, that book? I found it three days earlier. Around the same area I found you!", [
      { text: 'That\'s my book.', callback: () => this.bookMine() },
      { text: 'Where did you get that book?', callback: () => this.bookFound() }
    ], 'portrait-jin');
  }

  bookMine() {
    // Script: "Apologies! I actually found it in the same area I found you!"
    this.showDialogue('Jin', 'Apologies! I actually found it in the same area I found you!', [
      { text: 'Continue', callback: () => this.bookReturn() }
    ], 'portrait-jin');
  }

  bookFound() {
    this.showDialogue('Jin', "Oh, that book? I found it three days earlier. Around the same area I found you!", [
      { text: 'Continue', callback: () => this.bookReturn() }
    ], 'portrait-jin');
  }

  bookReturn() {
    // Script: River asks for it back, Jin gives it
    this.showDialogue('River', 'Could I have it back?', [
      { text: 'Continue', callback: () => {
        this.showDialogue('Jin', 'Of course!', [
          { text: 'Continue', callback: () => this.trekToCastle() }
        ], 'portrait-jin');
      }}
    ], 'portrait-river');
  }

  trekToCastle() {
    // Script: River treks candy land to reach the castle (walking if lost, horse if perfect)
    const travelText = this.gameState.hasHorse
      ? 'River rides Jin\'s horse through Candy Land toward the castle.'
      : 'River walks through Candy Land toward the castle. It\'s getting pretty late.';
    this.showDialogue('Narrator', travelText, [
      { text: 'Continue', callback: () => this.reachCastle() }
    ]);
  }

  reachCastle() {
    // Script: Two lollipop wielding guards stop River
    this.showDialogue('Guard 1', 'One of us speaks truth, the other-', [
      { text: 'Continue', callback: () => this.guardComedy() }
    ]);
  }

  guardComedy() {
    // Script: Guards break the fourth wall
    this.showDialogue('Guard 2', "You don't seem like you're from around here.", [
      { text: 'Continue', callback: () => this.guardComedy2() }
    ]);
  }

  guardComedy2() {
    this.showDialogue('Guard 2', "Oh we're sticking to the script? My bad. Yes, whatever he said.", [
      { text: 'Continue', callback: () => this.guardComedy3() }
    ]);
  }

  guardComedy3() {
    this.showDialogue('Guard 1', "You know what, I'm not even in the mood for it anymore.", [
      { text: 'Continue', callback: () => this.guardComedy4() }
    ]);
  }

  guardComedy4() {
    this.showDialogue('Guard 2', "No but we were gonna do it. I mean we have to do it.", [
      { text: 'Continue', callback: () => this.guardComedy5() }
    ]);
  }

  guardComedy5() {
    this.showDialogue('Guard 1', "Yeah well you ruined it.", [
      { text: 'Continue', callback: () => this.guardsFight() }
    ]);
  }

  guardsFight() {
    // Script: The guards fight, letting River pass through quietly
    this.showDialogue('Narrator', 'The guards start arguing and eventually start fighting each other. River quietly slips past them into the castle.', [
      { text: 'Enter the castle', callback: () => this.kandyKween() }
    ]);
  }

  kandyKween() {
    // Script: Kandy Kween "WHO GOES THERE..?" + lights turn on + Matt dialogue
    this.cameras.main.flash(300, 255, 200, 255);
    this.showDialogue('Kandy Kween', 'WHO GOES THERE..?', [
      { text: 'Continue', callback: () => this.kandyLightsOn() }
    ], 'portrait-kandy-kween');
  }

  kandyLightsOn() {
    // Script: Lights turn on, Kandy Kween asks Matt
    this.showDialogue('Kandy Kween', 'Matt, who is this?', [
      { text: 'Continue', callback: () => this.mattReply() }
    ], 'portrait-kandy-kween');
  }

  mattReply() {
    // Script: Matt's reply
    this.showDialogue('Matt', "I am unaware madame. Whoever it is, they made it through the guards and trials.", [
      { text: 'Continue', callback: () => this.kandyTrials() }
    ]);
  }

  kandyTrials() {
    // Script: Kandy Kween's response about making the trials herself
    this.showDialogue('Kandy Kween', "Well that isn't very reassuring, I made those myself.", [
      { text: 'Continue', callback: () => this.kandyDialogue() }
    ], 'portrait-kandy-kween');
  }

  kandyDialogue() {
    // Script: River notices the necklace, dialogue options about the puzzles
    this.showDialogue('Narrator', "River notices that Kandy Kween's necklace contains an essential ingredient for the literacy elixir.", [
      { text: 'Continue', callback: () => this.kandyChoice() }
    ]);
  }

  kandyChoice() {
    // Script: Three dialogue options about the puzzles
    this.showDialogue('Kandy Kween', 'So, what did you think of my guards and trials?', [
      { text: '"I thought they were great"', callback: () => this.kandyGreat() },
      { text: '"They were really easy"', callback: () => this.kandyEasy() },
      { text: '"Those puzzles sucked"', callback: () => this.kandySucked() }
    ], 'portrait-kandy-kween');
  }

  kandyGreat() {
    // Script: "Thanks, here's your reward..."
    this.showDialogue('Kandy Kween', 'Thanks, here is your reward...', [
      { text: 'Continue', callback: () => this.trapDoor() }
    ], 'portrait-kandy-kween');
  }

  kandyEasy() {
    // Script: Kandy Kween starts fuming "OH REALLY ??"
    this.cameras.main.shake(200, 0.01);
    this.showDialogue('Kandy Kween', 'OH REALLY ??', [
      { text: 'Continue', callback: () => this.trapDoor() }
    ], 'portrait-kandy-kween');
  }

  kandySucked() {
    // Script: "NOW WERE THEY? YOU KNOW WHAT ELSE SUCKS?" River: "... no"
    this.cameras.main.shake(300, 0.015);
    this.cameras.main.flash(200, 255, 100, 100);
    this.showDialogue('Kandy Kween', 'NOW WERE THEY? YOU KNOW WHAT ELSE SUCKS?', [
      { text: 'Continue', callback: () => {
        this.showDialogue('River', '...', [
          { text: 'Continue', callback: () => this.trapDoor() }
        ], 'portrait-river');
      }}
    ], 'portrait-kandy-kween');
  }

  trapDoor() {
    // Script: Kandy Kween activates trap door regardless of dialogue option
    this.cameras.main.shake(500, 0.03);
    audioManager.playScreenFlash();
    this.showDialogue('Narrator', 'Kandy Kween activates a trap door. River falls into darkness.', [
      { text: 'Continue', callback: () => this.candyCave() }
    ]);
  }

  candyCave() {
    // Script: River is dropped in candy cave that transitions into rock cave
    this.cameras.main.setBackgroundColor('#1a0a0a');
    this.showDialogue('Narrator', 'River plummets through a candy cave that transitions into an actual rock cave. They plummet into the water.', [
      { text: 'Continue', callback: () => this.underwater() }
    ]);
  }

  underwater() {
    // Script: Remains of candy people, light in depths
    this.cameras.main.setBackgroundColor('#050a1a');
    this.showDialogue('Narrator', 'River notices what surrounds them are remains of candy people. A light glows in the depths below.', [
      { text: 'Continue', callback: () => this.lightGrow() }
    ]);
  }

  lightGrow() {
    // Script: Light becomes bigger, River tries to swim back up
    this.showDialogue('Narrator', 'The light becomes bigger. River tries to swim back up but something grabs them.', [
      { text: 'Continue', callback: () => this.qte() }
    ]);
  }

  qte() {
    // Script: Quick-time event (regardless of winning you still get dragged down)
    this.showDialogue('Narrator', '[QUICK TIME EVENT] Something drags River down into the depths. No matter what they do, they cannot escape.', [
      { text: 'Continue', callback: () => this.pitchBlack() }
    ]);
  }

  pitchBlack() {
    // Script: Pitch black, eyes open, blurry lights
    this.cameras.main.setBackgroundColor('#000000');
    this.cameras.main.fadeOut(1000);
    this.cameras.main.once('camerafadeoutcomplete', () => {
      this.cameras.main.fadeIn(2000);
      this.cameras.main.setBackgroundColor('#050a1a');
      this.showDialogue('Narrator', "Pitch black. Then... eyelids open. Blurry lights. A shot of River's eye. River finds themselves in an underwater world.", [
        { text: 'Continue', callback: () => this.transitionTo('AquariusScene', 1500, 0x050a1a) }
      ]);
    });
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
