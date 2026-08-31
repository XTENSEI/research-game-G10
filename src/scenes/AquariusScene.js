import { BaseScene } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';
import { RiddleBox } from '../ui/RiddleBox.js';
import aquariusRiddleData from '../../riddles/aquarius-riddles.json';
import mazeData from '../../puzzles/aquarius-cave-maze.json';

const AQUARIUS_RIDDLES = aquariusRiddleData.riddles;
const DIRECTIONS = ['north', 'east', 'south', 'west'];
const DIR_LABELS = { north: 'North', east: 'East', south: 'South', west: 'West' };

export class AquariusScene extends BaseScene {
  constructor() {
    super({ key: 'AquariusScene', biomeId: 'aquarius' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1500);
    this.cameras.main.setBackgroundColor('#050a1a');

    this.setupGround(0x062a3a, 460);

    // Deep ocean parallax
    this.setupParallax([
      { key: 'particle', speed: 0.01, y: 0, depth: -5, tint: 0x0a1a3a, alpha: 0.6 },
      { key: 'particle', speed: 0.05, y: 80, depth: -4, tint: 0x1a3a5a, alpha: 0.4 },
      { key: 'particle', speed: 0.1, y: 200, depth: -3, tint: 0x228B22, alpha: 0.3 },
      { key: 'particle', speed: 0.2, y: 320, depth: -2, tint: 0x4682B4, alpha: 0.35 },
      { key: 'particle', speed: 0.4, y: 420, depth: -1, tint: 0xf4a460, alpha: 0.25 }
    ]);

    const player = this.createPlayer(100, 380);

    this.riddleBox = new RiddleBox(this);
    this.riddleQueue = [];

    // NPCs
    this.createNPC(330, 370, 'gillbert', 'Gillbert', 'portrait-gillbert', 0xd2a06a);
    this.createNPC(410, 370, 'scalleon', 'Scalleon', 'portrait-scalleon', 0x44dddd);
    this.createNPC(620, 370, 'market-lady', 'Market Lady', 'portrait-market-lady', 0xff8a5a);

    // City Hall facing east (environmental clue)
    const hall = this.add.rectangle(830, 400, 120, 160, 0x2a4a6a, 1).setDepth(6);
    hall.setStrokeStyle(2, 0x6ab0e0, 0.6);
    this.add.text(830, 340, 'CITY HALL', {
      fontFamily: 'Arial, sans-serif', fontSize: '11px', color: '#6ab0e0',
      backgroundColor: '#00000055', padding: { x: 6, y: 2 }
    }).setOrigin(0.5).setDepth(7);
    this.add.rectangle(850, 375, 10, 8, 0x66ccff, 0.8).setDepth(7).setStrokeStyle(1, 0x000000, 0.4);
    this.add.rectangle(810, 375, 10, 8, 0x66ccff, 0.8).setDepth(7).setStrokeStyle(1, 0x000000, 0.4);

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

    // Fish particles
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

    this.setupVignette(0.5, 89);

    audioManager.playAmbientAquarius();

    this.gillbertTalked = false;
    this.hallNoted = false;
    this.caveBlocked = false;
    const gillbert = this.npcGroup.getChildren().find(n => n.npcKey === 'gillbert');
    const lady = this.npcGroup.getChildren().find(n => n.npcKey === 'market-lady');
    this.physics.add.overlap(player, gillbert, () => this.gillbertTalk());
    this.physics.add.overlap(player, lady, () => this.marketLadyTalk());

    // City hall approach = environmental clue
    this.hallZone = this.add.zone(830, 380, 140, 60).setDepth(3);
    this.physics.add.existing(this.hallZone, true);
    this.physics.add.overlap(player, this.hallZone, () => {
      if (!this.hallNoted && !this.dialogueActive && !this.mazeActive) {
        this.hallNoted = true;
        this.showDialogue('Narrator',
          'City Hall faces east. The fish folk built every road to answer its gaze.',
          [{ text: 'Good to know', callback: () => {} }]
        );
      }
    });

    // Cave entrance (right side) - gated by >=1 riddle
    this.caveZone = this.add.zone(920, 380, 90, 140).setDepth(3);
    this.physics.add.existing(this.caveZone, true);
    this.caveArrow = this.add.text(930, 320, 'CAVE \u25B8', {
      fontFamily: 'Arial, sans-serif', fontSize: '13px', color: '#66ccff',
      backgroundColor: '#00000066', padding: { x: 8, y: 4 }
    }).setOrigin(0.5).setDepth(7);
    this.tweens.add({ targets: this.caveArrow, alpha: { from: 0.4, to: 1 }, duration: 800, yoyo: true, repeat: -1 });
    this.physics.add.overlap(player, this.caveZone, () => {
      if (!this.dialogueActive && !this.mazeActive) this.tryEnterCave();
    });
  }

  // ─── NPC Clues ─────────────────────────────────────────

  gillbertTalk() {
    if (this.gillbertTalked || this.dialogueActive || this.mazeActive) return;
    this.gillbertTalked = true;
    this.showDialogue('Gillbert',
      'Hey, do you know where wild sea berries grow? They are the sweetest thing in Aquarius, and ailing folk swear by them.',
      [
        { text: 'West?', callback: () => this.replyWest() },
        { text: 'North?', callback: () => this.replyNorth() },
        { text: 'Ask about the cave', callback: () => this.gillbertCave() }
      ],
      'portrait-gillbert'
    );
  }

  replyWest() {
    this.showDialogue('Scalleon',
      "Hmm, I believe it's in the west, I think? The locals claim so at least. But the freshest ones I ever tasted came from the north, near the market.",
      [{ text: 'Continue', callback: () => this.gillbertRiddles() }],
      'portrait-scalleon'
    );
  }

  replyNorth() {
    this.showDialogue('Scalleon',
      "No, real ones grow from the north. The market lady told me. But the locals claim west. Hard to say who's pulling whose fin.",
      [{ text: 'Continue', callback: () => this.gillbertRiddles() }],
      'portrait-scalleon'
    );
  }

  gillbertCave() {
    this.showDialogue('Gillbert',
      'The Grave Cave Maze of Craze? It is forbidden for citizens. The council scribbled directional riddles on a tablet so only the truly literate may pass. Reckon that includes you?',
      [{ text: 'Reckon it does', callback: () => this.gillbertRiddles() }],
      'portrait-gillbert'
    );
  }

  gillbertRiddles() {
    this.showDialogue('Gillbert',
      'Before you crawl off into that dark, answer a riddle or two. A wandering mind gets swallowed whole down there.',
      [{ text: 'Answer the riddles', callback: () => this.aquariusRiddles() }],
      'portrait-gillbert'
    );
  }

  marketLadyTalk() {
    if (this.dialogueActive || this.mazeActive || this.ladyTalked) return;
    this.ladyTalked = true;
    this.showDialogue('Market Lady',
      'Fresh sea berries, young one? Real ones grow from the North. Follow the red needle and you will find the sweetest patch.',
      [{ text: 'North, got it', callback: () => {} }],
      'portrait-market-lady'
    );
  }

  // ─── Riddles ────────────────────────────────────────────

  aquariusRiddles() {
    const done = this.gameState.completedRiddles || [];
    this.riddleQueue = AQUARIUS_RIDDLES.filter(r => !done.includes(r.id));
    this.askNextRiddle();
  }

  askNextRiddle() {
    if (this.riddleQueue.length === 0) { this.caveIntro(); return; }
    const r = this.riddleQueue.shift();
    this.riddleBox.show({
      title: 'Aquarius Riddle',
      question: r.question,
      options: r.options,
      correct: r.correct,
      explanation: r.explanation,
      onComplete: (ok) => {
        const done = this.gameState.completedRiddles || (this.gameState.completedRiddles = []);
        if (!done.includes(r.id)) done.push(r.id);
        this.time.delayedCall(300, () => {
          if (this.riddleQueue.length > 0) {
            this.showDialogue('Gillbert', 'And another, if you would be so bold? Or straight to the cave?', [
              { text: 'Answer another', callback: () => this.askNextRiddle() },
              { text: 'To the cave', callback: () => this.caveIntro() }
            ], 'portrait-gillbert');
          } else {
            this.showDialogue('Gillbert', 'Sharp as a piranha\'s tooth. The cave is yours to try.', [
              { text: 'To the cave', callback: () => this.caveIntro() }
            ], 'portrait-gillbert');
          }
        });
      }
    });
  }

  // ─── Cave Maze ──────────────────────────────────────────

  tryEnterCave() {
    const riddles = this.gameState.completedRiddles || [];
    const anyDone = riddles.length > 0;
    if (anyDone) { this.caveIntro(); return; }
    if (this.caveBlocked) return;
    this.caveBlocked = true;
    this.showDialogue('Tablet',
      'A low hum answers. \u201CProve your understanding before you enter.\u201D Gillbert likely knows what that means.',
      [{ text: 'Ask Gillbert', callback: () => this.gillbertRiddles() }]
    );
  }

  caveIntro() {
    if (this.mazeActive) return;
    this.mazeActive = true;
    this.showDialogue('Narrator',
      'River reaches the Grave Cave Maze of Craze. A stone tablet glows with directional riddles, guarding the deep path.',
      [{ text: 'Read the tablet', callback: () => this.caveMaze() }],
      'portrait-river'
    );
  }

  caveMaze() {
    const dirs = this.shuffle([...DIRECTIONS]);
    this.mazeSeq = dirs.map(d => {
      const qs = mazeData.directions[d].questions;
      return { dir: d, question: qs[Math.floor(Math.random() * qs.length)] };
    });
    this.mazeIndex = 0;
    this.askMazeQuestion();
  }

  askMazeQuestion() {
    if (this.mazeIndex >= this.mazeSeq.length) { this.mazeComplete(); return; }
    const q = this.mazeSeq[this.mazeIndex];
    this.mazeUI(q.question, (pick) => {
      if (pick === q.dir) {
        audioManager.playCorrect();
        this.cameras.main.flash(200, 100, 200, 255, false);
        this.mazeIndex++;
        this.time.delayedCall(450, () => this.askMazeQuestion());
      } else {
        // Script: failing sends you back to the last question, no visual cue
        audioManager.playWrong();
        this.mazeIndex = Math.max(0, this.mazeIndex - 1);
        this.time.delayedCall(450, () => this.askMazeQuestion());
      }
    });
  }

  mazeUI(question, onPick) {
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;

    const overlay = this.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.8).setDepth(120).setScrollFactor(0);
    const panel = this.add.graphics().setDepth(121).setScrollFactor(0);
    panel.fillStyle(0x06101f, 0.97);
    panel.fillRoundedRect(80, 40, w - 160, 150, 12);
    panel.lineStyle(2, 0x66ccff, 0.7);
    panel.strokeRoundedRect(80, 40, w - 160, 150, 12);

    const title = this.add.text(w / 2, 60, 'THE GRAVE CAVE MAZE', {
      fontFamily: '"Courier New", monospace', fontSize: '15px', fontStyle: 'bold', color: '#66ccff'
    }).setOrigin(0.5).setDepth(122).setScrollFactor(0);

    const qText = this.add.text(w / 2, 105, question, {
      fontFamily: '"Courier New", monospace', fontSize: '16px', color: '#ffffff',
      wordWrap: { width: w - 220 }, align: 'center', lineSpacing: 5
    }).setOrigin(0.5).setDepth(122).setScrollFactor(0);

    const progress = this.add.text(w / 2, 235, `Chambers cleared: ${this.mazeIndex} / ${this.mazeSeq.length}`, {
      fontFamily: '"Courier New", monospace', fontSize: '13px', color: '#5588aa'
    }).setOrigin(0.5).setDepth(122).setScrollFactor(0);

    const startX = w / 2;
    const startY = 300;
    const gap = 210;
    const positions = [
      { dx: 0, dy: 0 },        // North
      { dx: gap, dy: 0 },      // East
      { dx: -gap, dy: 0 },     // West
      { dx: 0, dy: 90 }        // South
    ];
    const btnEls = [];
    DIRECTIONS.forEach((dir, i) => {
      const bx = startX + positions[i].dx;
      const by = startY + positions[i].dy;
      const bg = this.add.graphics().setDepth(121).setScrollFactor(0);
      bg.fillStyle(0x0a2440, 0.95);
      bg.fillRoundedRect(bx - 70, by - 28, 140, 56, 10);
      bg.lineStyle(1, 0x4499cc, 0.5);
      bg.strokeRoundedRect(bx - 70, by - 28, 140, 56, 10);
      const label = this.add.text(bx, by, DIR_LABELS[dir], {
        fontFamily: '"Courier New", monospace', fontSize: '16px', fontStyle: 'bold', color: '#9cc'
      }).setOrigin(0.5).setDepth(122).setScrollFactor(0);
      const zone = this.add.zone(bx, by, 140, 56).setDepth(123).setScrollFactor(0).setInteractive({ useHandCursor: true });
      zone.on('pointerover', () => {
        bg.clear();
        bg.fillStyle(0x0a3a5a, 0.98);
        bg.fillRoundedRect(bx - 70, by - 28, 140, 56, 10);
        bg.lineStyle(2, 0x66ccff, 1);
        bg.strokeRoundedRect(bx - 70, by - 28, 140, 56, 10);
        label.setColor('#66ccff');
      });
      zone.on('pointerout', () => {
        bg.clear();
        bg.fillStyle(0x0a2440, 0.95);
        bg.fillRoundedRect(bx - 70, by - 28, 140, 56, 10);
        bg.lineStyle(1, 0x4499cc, 0.5);
        bg.strokeRoundedRect(bx - 70, by - 28, 140, 56, 10);
        label.setColor('#9cc');
      });
      zone.on('pointerdown', () => {
        const pick = dir;
        btnEls.forEach(el => { if (el && el.destroy) el.destroy(); });
        [overlay, panel, title, qText, progress].forEach(el => el.destroy());
        onPick(pick);
      });
      btnEls.push(bg, label, zone);
    });
  }

  mazeComplete() {
    this.mazeActive = false;
    const puzzles = this.gameState.completedPuzzles || (this.gameState.completedPuzzles = []);
    if (!puzzles.includes('cave-maze')) puzzles.push('cave-maze');
    const inv = this.gameState.inventory || (this.gameState.inventory = []);
    if (!inv.includes('aquarius-ingredient')) inv.push('aquarius-ingredient');

    audioManager.playScreenFlash();
    this.cameras.main.flash(500, 100, 220, 255);
    this.showDialogue('Narrator',
      'The final chamber opens into a grove of wild sea berries, glowing faintly under the water\'s dark. River gathers the freshest handful: the second ingredient of the Gift of Education.',
      [{ text: 'Gather the berries', callback: () => this.transitionTo('CosmaScene', 1500, 0x050510) }],
      'portrait-river'
    );
  }

  update() {
    this.handleMovement(160);
    this.updateParallax();
  }
}
