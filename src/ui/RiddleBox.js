import Phaser from 'phaser';
import { audioManager } from '../audio/AudioManager.js';

/**
 * RiddleBox - One-try multiple choice riddle with explanation feedback.
 *
 * Locks scene input (scene.dialogueActive = true) while active.
 * onComplete(isCorrect) fires after the player dismisses the feedback.
 */
export class RiddleBox {
  constructor(scene) {
    this.scene = scene;
    this.state = 'IDLE'; // IDLE | ACTIVE
    this.elements = [];
    this.optionEls = [];
    this.keyCleanup = [];
    this.onComplete = null;
    this.ac = scene.uiAccent || '#ffdd00';
    this.acNum = parseInt(this.ac.replace('#', ''), 16);
  }

  show({ title = 'Riddle', question, options, correct, explanation, onComplete }) {
    if (this.state !== 'IDLE') return;
    this.state = 'ACTIVE';
    this.onComplete = onComplete || null;
    this.scene.dialogueActive = true;
    this._build(title, question, options, correct, explanation);
  }

  _build(title, question, options, correct, explanation) {
    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;

    const overlay = this.scene.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.78)
      .setDepth(120).setScrollFactor(0);
    this.elements.push(overlay);

    const panel = this.scene.add.graphics().setDepth(121).setScrollFactor(0);
    panel.fillStyle(0x0d0d2a, 0.97);
    panel.fillRoundedRect(80, 28, w - 160, 158, 12);
    panel.lineStyle(2, this.acNum, 0.7);
    panel.strokeRoundedRect(80, 28, w - 160, 158, 12);
    this.elements.push(panel);

    const titleText = this.scene.add.text(w / 2, 46, `\u2726 ${title} \u2726`, {
      fontFamily: '"Courier New", monospace', fontSize: '15px', fontStyle: 'bold',
      color: this.ac
    }).setOrigin(0.5).setDepth(122).setScrollFactor(0);
    this.elements.push(titleText);

    const qText = this.scene.add.text(w / 2, 96, question, {
      fontFamily: '"Courier New", monospace', fontSize: '16px', color: '#ffffff',
      wordWrap: { width: w - 220 }, align: 'center', lineSpacing: 5
    }).setOrigin(0.5).setDepth(122).setScrollFactor(0);
    this.elements.push(qText);

    // ── Options ──
    const startY = 212;
    options.forEach((opt, i) => {
      const by = startY + i * 52;
      const bg = this.scene.add.graphics().setDepth(121).setScrollFactor(0);
      bg.fillStyle(0x1a1a3a, 0.95);
      bg.fillRoundedRect(w / 2 - 320, by, 640, 44, 8);
      bg.lineStyle(1, 0x4444aa, 0.5);
      bg.strokeRoundedRect(w / 2 - 320, by, 640, 44, 8);

      const label = this.scene.add.text(w / 2, by + 22, `${String.fromCharCode(65 + i)}. ${opt}`, {
        fontFamily: '"Courier New", monospace', fontSize: '15px', color: '#999999'
      }).setOrigin(0.5).setDepth(122).setScrollFactor(0);

      const zone = this.scene.add.zone(w / 2, by + 22, 640, 44)
        .setDepth(123).setScrollFactor(0).setInteractive({ useHandCursor: true });
      zone.on('pointerover', () => {
        bg.clear();
        bg.fillStyle(0x2a1a4a, 0.98);
        bg.fillRoundedRect(w / 2 - 320, by, 640, 44, 8);
        bg.lineStyle(2, this.acNum, 1);
        bg.strokeRoundedRect(w / 2 - 320, by, 640, 44, 8);
        label.setColor(this.ac);
      });
      zone.on('pointerout', () => {
        bg.clear();
        bg.fillStyle(0x1a1a3a, 0.95);
        bg.fillRoundedRect(w / 2 - 320, by, 640, 44, 8);
        bg.lineStyle(1, 0x4444aa, 0.5);
        bg.strokeRoundedRect(w / 2 - 320, by, 640, 44, 8);
        label.setColor('#999999');
      });
      zone.on('pointerdown', () => this._pick(i, options, correct, explanation));
      this.optionEls.push(bg, label, zone);
    });

    // Number keys 1-4
    options.forEach((opt, i) => {
      const handler = () => this._pick(i, options, correct, explanation);
      this.scene.input.keyboard.once(`keydown-${i + 1}`, handler);
      this.keyCleanup.push({ code: `keydown-${i + 1}`, handler });
    });
  }

  _pick(i, options, correct, explanation) {
    if (this.state !== 'ACTIVE') return;
    this.state = 'FEEDBACK';

    // Clean up keyboard shortcuts + option buttons
    this.keyCleanup.forEach(({ code, handler }) => this.scene.input.keyboard.off(code, handler));
    this.keyCleanup = [];
    this.optionEls.forEach(el => { if (el && el.destroy) { el.disableInteractive(); el.destroy(); } });
    this.optionEls = [];

    const isCorrect = i === correct;
    if (isCorrect) audioManager.playCorrect();
    else { audioManager.playWrong(); this.scene.cameras.main.shake(200, 0.012); }

    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;

    const fb = this.scene.add.text(w / 2, 195, isCorrect ? 'CORRECT!' : 'INCORRECT', {
      fontFamily: '"Courier New", monospace', fontSize: '32px', fontStyle: 'bold',
      color: isCorrect ? '#00ff66' : '#ff5555', stroke: '#000000', strokeThickness: 4
    }).setOrigin(0.5).setDepth(124).setScrollFactor(0);
    this.elements.push(fb);

    const exText = this.scene.add.text(w / 2, 250, explanation, {
      fontFamily: '"Courier New", monospace', fontSize: '15px', color: '#cccccc',
      wordWrap: { width: w - 240 }, align: 'center', lineSpacing: 4
    }).setOrigin(0.5).setDepth(124).setScrollFactor(0);
    this.elements.push(exText);

    const btn = this.scene.add.rectangle(w / 2, h - 60, 220, 46, this.acNum, 0.15)
      .setStrokeStyle(2, this.acNum, 0.8)
      .setDepth(124).setScrollFactor(0).setInteractive({ useHandCursor: true });
    const btnText = this.scene.add.text(w / 2, h - 60, 'Continue', {
      fontFamily: '"Courier New", monospace', fontSize: '16px', color: this.ac
    }).setOrigin(0.5).setDepth(125).setScrollFactor(0);
    this.elements.push(btn, btnText);

    btn.on('pointerdown', () => {
      const cb = this.onComplete;
      this.destroy();
      if (cb) cb(isCorrect);
    });
    btn.on('pointerover', () => btn.setFillStyle(this.acNum, 0.35));
    btn.on('pointerout', () => btn.setFillStyle(this.acNum, 0.15));

    // Space/Enter also advance
    const advance = () => {
      const cb = this.onComplete;
      this.destroy();
      if (cb) cb(isCorrect);
    };
    this.scene.input.keyboard.once('keydown-SPACE', advance);
    this.scene.input.keyboard.once('keydown-ENTER', advance);
    this.keyCleanup.push({ code: 'keydown-SPACE', handler: advance }, { code: 'keydown-ENTER', handler: advance });
  }

  destroy() {
    this.state = 'IDLE';
    this.keyCleanup.forEach(({ code, handler }) => this.scene.input.keyboard.off(code, handler));
    this.keyCleanup = [];
    this.elements.forEach(el => { if (el && el.destroy) { el.disableInteractive(); el.destroy(); } });
    this.elements = [];
    this.optionEls = [];
    this.onComplete = null;
    this.scene.dialogueActive = false;
  }
}
