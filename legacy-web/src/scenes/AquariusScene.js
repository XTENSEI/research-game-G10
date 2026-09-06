import { BaseScene, GROUND_Y } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';
import { RiddleBox } from '../ui/RiddleBox.js';
import aquariusRiddleData from '../../riddles/aquarius-riddles.json';
import mazeData from '../../puzzles/aquarius-cave-maze.json';

const AQUARIUS_RIDDLES = aquariusRiddleData.riddles;
const DIRECTIONS = ['north', 'east', 'south', 'west'];
const DIR_LABELS = { north: 'North', east: 'East', south: 'South', west: 'West' };
const AQUA_Y = GROUND_Y - 56;

export class AquariusScene extends BaseScene {
  constructor() {
    super({ key: 'AquariusScene', biomeId: 'aquarius' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1500);
    this.cameras.main.setBackgroundColor('#052030');
    this.setTheme('#6ad4ff', '#17577a');

    this.setupSky('sky-aqua');
    this.setupGround('tile-aqua', GROUND_Y, 0x0a2436);

    // Distant seabed ridge + kelp silhouettes
    this.addBand('ridge-aqua', 470, 200, -5, 0.95, 0.05);
    this.add.image(180, 330, 'kelp').setScrollFactor(0).setDepth(-4).setAlpha(0.8).setScale(0.9);
    this.add.image(760, 300, 'kelp').setScrollFactor(0).setDepth(-4).setAlpha(0.7);

    // Shipwreck on the far left, half-buried
    const wreck = this.add.image(120, GROUND_Y, 'shipwreck').setScrollFactor(0).setDepth(5).setScale(0.85);
    this.add.image(120, GROUND_Y - 14, 'rock').setScrollFactor(0).setDepth(6).setAlpha(0.9);

    // Corals + seaweed lining the sea floor
    this.placeProp('coral', 330, 1.0, 7);
    this.placeProp('seaweed', 720, 1.1, 7);
    this.placeProp('coral', 560, 0.8, 7);
    this.placeProp('seaweed', 950, 0.85, 7);
    this.placeProp('coral', 210, 0.9, 7);

    // Light shafts from the surface
    for (let i = 0; i < 4; i++) {
      const ray = this.add.rectangle(120 + i * 260, 150, 50, 700, 0x9fe0ff, 0.05)
        .setDepth(0).setScrollFactor(0).setAngle(6 + i * 4);
      this.tweens.add({
        targets: ray, alpha: { from: 0.03, to: 0.07 }, x: ray.x + 20,
        duration: 4000 + i * 900, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
      });
    }

    const player = this.createPlayer(90, AQUA_Y);

    this.riddleBox = new RiddleBox(this);
    this.riddleQueue = [];

    // NPCs on the sea floor
    this.createNPC(330, AQUA_Y, 'gillbert', 'Gillbert', 'portrait-gillbert');
    const scalleon = this.createNPC(415, AQUA_Y, 'scalleon', 'Scalleon', 'portrait-scalleon');
    scalleon.talkable = false;
    this.createNPC(630, AQUA_Y, 'market-lady', 'Market Lady', 'portrait-market-lady');

    // City Hall facing east (environmental clue) — stone dome building
    const hallX = 830;
    const hall = this.add.container(hallX, 0).setDepth(6).setScrollFactor(0);
    const hg = this.add.graphics();
    hg.fillStyle(0x35647a, 1);
    hg.fillRoundedRect(-70, -190, 140, 190, 8);
    hg.fillStyle(0x2a4f66, 1);
    hg.fillTriangle(-70, -190, 70, -190, 0, -250); // dome
    hg.lineStyle(2, 0x6ab0e0, 0.7);
    hg.strokeRoundedRect(-70, -190, 140, 190, 8);
    hall.add(hg);
    // dome windows row + doorway facing EAST (right)
    hall.add(this.add.rectangle(-30, -160, 22, 16, 0xaef0ff, 0.9).setStrokeStyle(1, 0x000000, 0.4));
    hall.add(this.add.rectangle(30, -160, 22, 16, 0xaef0ff, 0.9).setStrokeStyle(1, 0x000000, 0.4));
    hall.add(this.add.rectangle(58, -90, 16, 34, 0x1a3242, 0.9).setStrokeStyle(1, 0x6ab0e0, 0.6));
    // pillar accents
    hall.add(this.add.rectangle(-55, -60, 10, 60, 0x4a7a90, 1));
    hall.add(this.add.rectangle(55, -60, 10, 60, 0x4a7a90, 1));
    hall.add(this.add.text(0, -208, 'CITY HALL', {
      fontFamily: '"Courier New", monospace', fontSize: '11px', color: '#9fdcff',
      backgroundColor: '#00000066', padding: { x: 6, y: 2 }
    }).setOrigin(0.5));
    this.tweens.add({
      targets: this.add.circle(hallX + 62, -30, 8, 0x66ccff, 0.25).setScrollFactor(0).setDepth(7),
      scaleX: 1.6, scaleY: 1.6, alpha: 0, duration: 1600, repeat: -1
    });
    hall.setY(GROUND_Y);

    // Rising bubbles
    this.setupEmitter('bubble', {
      y: { min: 576, max: 576 },
      lifespan: 6000,
      speed: { min: 20, max: 50 },
      scale: { start: 0.2, end: 0.6 },
      alpha: { start: 0.4, end: 0.05 },
      frequency: 140,
      blendMode: 'ADD',
      tint: 0x88ccff,
      depth: 12
    });

    // Slow fish silhouettes crossing
    this.setupEmitter('particle', {
      x: { min: -60, max: -60 },
      y: { min: 120, max: 380 },
      lifespan: 9000,
      speed: { min: 40, max: 70 },
      angle: { min: 174, max: 186 },
      scale: { start: 0.5, end: 0.2 },
      alpha: { start: 0.35, end: 0.05 },
      frequency: 900,
      tint: [0xffaa66, 0x77ccff, 0xff88aa],
      depth: 3
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
    this.hallZone = this.add.zone(hallX, AQUA_Y, 160, 80).setDepth(3);
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
    this.caveZone = this.add.zone(950, AQUA_Y, 110, 120).setDepth(3);
    this.physics.add.existing(this.caveZone, true);
    this.add.circle(940, GROUND_Y - 26, 46, 0x071018, 1).setScrollFactor(0).setDepth(6).setStrokeStyle(3, 0x1a3242, 1);
    this.caveArrow = this.add.text(930, 260, 'CAVE \u25B8', {
      fontFamily: '"Courier New", monospace', fontSize: '13px', color: '#9fdcff',
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
    this.updateNPCs();
  }
}
