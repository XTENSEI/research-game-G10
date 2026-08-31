import Phaser from 'phaser';
import { audioManager } from '../audio/AudioManager.js';

/**
 * PunctuationGate - Fill-in-the-blank punctuation puzzle.
 *
 * Sentence-by-sentence. Each blank is a clickable chip; picking a blank opens
 * its options. Player advances sentence by sentence, then checks the gate.
 * Multiple tries allowed (marks wrong blanks and lets the player re-edit).
 *
 * Locks scene input while active. onComplete() fires on success.
 */
function renderSentenceText(text, blanks, selections) {
  const positions = blanks
    .map((b, i) => ({ pos: b.position, i }))
    .sort((a, b) => a.pos - b.pos);
  let out = '';
  let idx = 0;
  for (const { pos, i } of positions) {
    out += text.slice(idx, pos);
    out += selections[i] ? `[${selections[i]}]` : '[ _ ]';
    idx = pos + Math.max(1, blanks[i].answer.length);
  }
  out += text.slice(idx);
  return out;
}

export class PunctuationGate {
  constructor(scene) {
    this.scene = scene;
    this.state = 'IDLE'; // IDLE | ACTIVE
    this.elements = [];
    this.sentenceEls = [];
    this.frameEls = [];
    this.onComplete = null;
  }

  show({ title, sentences, onComplete }) {
    if (this.state !== 'IDLE') return;
    this.state = 'ACTIVE';
    this.scene.dialogueActive = true;
    this.sentences = sentences;
    this.selections = sentences.map(s => s.blanks.map(() => null));
    this.index = 0;
    this.optionRowEls = [];
    this.wrongBlanks = {};
    this.onComplete = onComplete || null;
    this._buildFrame(title);
    this._renderSentence();
  }

  _buildFrame(title) {
    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;

    const overlay = this.scene.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.82)
      .setDepth(120).setScrollFactor(0);
    this.frameEls.push(overlay);

    const titleText = this.scene.add.text(w / 2, 28, title, {
      fontFamily: '"Georgia", serif', fontSize: '19px', fontStyle: 'bold',
      color: '#ffdd00', stroke: '#000000', strokeThickness: 3
    }).setOrigin(0.5).setDepth(122).setScrollFactor(0);
    this.frameEls.push(titleText);
  }

  _renderSentence() {
    this._clearSentenceArea();
    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;
    const s = this.sentences[this.index];
    const blanks = s.blanks;
    const sel = this.selections[this.index];

    // Progress
    const progress = this.scene.add.text(w / 2, 54, `Sentence ${this.index + 1} of ${this.sentences.length}`, {
      fontFamily: '"Courier New", monospace', fontSize: '13px', color: '#8888cc'
    }).setOrigin(0.5).setDepth(122).setScrollFactor(0);
    this.sentenceEls.push(progress);

    // Sentence display panel
    const panel = this.scene.add.graphics().setDepth(121).setScrollFactor(0);
    panel.fillStyle(0x0d0d2a, 0.97);
    panel.fillRoundedRect(60, 78, w - 120, 160, 12);
    panel.lineStyle(2, 0xffdd00, 0.7);
    panel.strokeRoundedRect(60, 78, w - 120, 160, 12);
    this.sentenceEls.push(panel);

    const sText = this.scene.add.text(w / 2, 158, renderSentenceText(s.text, blanks, sel), {
      fontFamily: '"Georgia", serif', fontSize: '17px', color: '#ffffff',
      wordWrap: { width: w - 200 }, align: 'center', lineSpacing: 6
    }).setOrigin(0.5).setDepth(122).setScrollFactor(0);
    this.sentenceEls.push(sText);

    // Blank chips
    const chipY = 268;
    const spacing = Math.min(180, (w - 160) / blanks.length);
    blanks.forEach((b, i) => {
      const cx = w / 2 + (i - (blanks.length - 1) / 2) * spacing;
      const marked = this.wrongBlanks[`${this.index}:${i}`];
      const label = sel[i] ? `Blank ${i + 1}: [${sel[i]}]` : `Blank ${i + 1}: [ _ ]`;
      const chip = this.scene.add.text(cx, chipY, label, {
        fontFamily: '"Courier New", monospace', fontSize: '14px',
        color: marked ? '#ff8888' : (sel[i] ? '#ffdd00' : '#dddddd'),
        backgroundColor: marked ? '#3a1a1a' : '#1a1a3a', padding: { x: 14, y: 7 }
      }).setOrigin(0.5).setDepth(122).setScrollFactor(0).setInteractive({ useHandCursor: true });
      chip.on('pointerdown', () => this._openOptions(i));
      chip.on('pointerover', () => chip.setBackgroundColor(marked ? '#4a2a2a' : '#2a1a4a'));
      chip.on('pointerout', () => chip.setBackgroundColor(marked ? '#3a1a1a' : '#1a1a3a'));
      this.sentenceEls.push(chip);
    });

    // ── Nav buttons ──
    const backBtn = this.scene.add.rectangle(170, h - 56, 110, 42, 0x333366, 0.6)
      .setDepth(124).setScrollFactor(0).setInteractive({ useHandCursor: true });
    const backText = this.scene.add.text(170, h - 56, '\u2039 Back', {
      fontFamily: '"Courier New", monospace', fontSize: '15px', color: '#cccccc'
    }).setOrigin(0.5).setDepth(125).setScrollFactor(0);
    if (this.index === 0) { backBtn.setAlpha(0.3); backBtn.disableInteractive(); }
    backBtn.on('pointerdown', () => { audioManager.playChoiceSelect(); this.wrongBlanks = {}; this.index--; this._renderSentence(); });
    this.sentenceEls.push(backBtn, backText);

    const isLast = this.index === this.sentences.length - 1;
    const filled = blanks.every((_, i) => sel[i] !== null);
    const navLabel = isLast ? 'Check the Gate' : 'Next \u203A';
    const nextBtn = this.scene.add.rectangle(w - 180, h - 56, 190, 42, 0xffdd00, 0.2)
      .setDepth(124).setScrollFactor(0).setInteractive({ useHandCursor: true });
    const nextText = this.scene.add.text(w - 180, h - 56, navLabel, {
      fontFamily: '"Courier New", monospace', fontSize: '15px', fontStyle: 'bold', color: '#ffdd00'
    }).setOrigin(0.5).setDepth(125).setScrollFactor(0);
    if (!filled) { nextBtn.setAlpha(0.4); nextBtn.disableInteractive(); }
    nextBtn.on('pointerdown', () => {
      if (isLast) { this._check(); }
      else { audioManager.playChoiceSelect(); this.wrongBlanks = {}; this.index++; this._renderSentence(); }
    });
    this.sentenceEls.push(nextBtn, nextText);
  }

  _openOptions(i) {
    this._clearOptions();
    const w = this.scene.cameras.main.width;
    const blanks = this.sentences[this.index].blanks;
    const b = blanks[i];
    const spacing = Math.min(220, (w - 200) / b.options.length);
    b.options.forEach((opt, oi) => {
      const bx = w / 2 + (oi - (b.options.length - 1) / 2) * spacing;
      const btn = this.scene.add.rectangle(bx, 330, 200, 46, 0x1a1a3a, 0.95)
        .setDepth(124).setScrollFactor(0).setInteractive({ useHandCursor: true })
        .setStrokeStyle(1, 0x8888aa, 0.6);
      const label = this.scene.add.text(bx, 330, opt, {
        fontFamily: '"Courier New", monospace', fontSize: '20px', color: '#ffdd00'
      }).setOrigin(0.5).setDepth(125).setScrollFactor(0);
      btn.on('pointerdown', () => {
        this.selections[this.index][i] = opt;
        delete this.wrongBlanks[`${this.index}:${i}`];
        audioManager.playChoiceSelect();
        this._clearOptions();
        this._renderSentence();
      });
      btn.on('pointerover', () => btn.setStrokeStyle(2, 0xffdd00, 1));
      btn.on('pointerout', () => btn.setStrokeStyle(1, 0x8888aa, 0.6));
      this.optionRowEls.push(btn, label);
    });
    const hint = this.scene.add.text(w / 2, 392, 'Choose the correct punctuation mark', {
      fontFamily: '"Courier New", monospace', fontSize: '13px', color: '#8888cc'
    }).setOrigin(0.5).setDepth(124).setScrollFactor(0);
    this.optionRowEls.push(hint);
  }

  _clearOptions() {
    this.optionRowEls.forEach(el => { if (el && el.destroy) { el.disableInteractive(); el.destroy(); } });
    this.optionRowEls = [];
  }

  _check() {
    let wrong = [];
    this.wrongBlanks = {};
    this.sentences.forEach((s, si) => {
      s.blanks.forEach((b, bi) => {
        if (this.selections[si][bi] !== b.answer) {
          wrong.push({ si, bi });
          this.wrongBlanks[`${si}:${bi}`] = true;
        }
      });
    });

    if (wrong.length === 0) {
      audioManager.playCorrect();
      this.scene.cameras.main.flash(300, 100, 255, 100, false);
      const cb = this.onComplete;
      this.destroy();
      if (cb) cb();
    } else {
      audioManager.playWrong();
      this.scene.cameras.main.shake(220, 0.012);
      this._showFeedback(wrong);
    }
  }

  _showFeedback(wrong) {
    this._clearSentenceArea();
    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;

    const fb = this.scene.add.text(w / 2, 200, `${wrong.length} blank${wrong.length > 1 ? 's' : ''} wrong!`, {
      fontFamily: '"Courier New", monospace', fontSize: '30px', fontStyle: 'bold',
      color: '#ff5555', stroke: '#000000', strokeThickness: 4
    }).setOrigin(0.5).setDepth(124).setScrollFactor(0);
    this.sentenceEls.push(fb);

    const sub = this.scene.add.text(w / 2, 250, 'The gate hums and stays shut. Fix the marked blanks and try again.', {
      fontFamily: '"Courier New", monospace', fontSize: '15px', color: '#cccccc',
      wordWrap: { width: w - 240 }, align: 'center'
    }).setOrigin(0.5).setDepth(124).setScrollFactor(0);
    this.sentenceEls.push(sub);

    const btn = this.scene.add.rectangle(w / 2, h - 56, 220, 46, 0xff5555, 0.15)
      .setStrokeStyle(2, 0xff5555, 0.8)
      .setDepth(124).setScrollFactor(0).setInteractive({ useHandCursor: true });
    const btnText = this.scene.add.text(w / 2, h - 56, 'Fix my answers', {
      fontFamily: '"Courier New", monospace', fontSize: '16px', color: '#ff8888'
    }).setOrigin(0.5).setDepth(125).setScrollFactor(0);
    this.sentenceEls.push(btn, btnText);

    const jumpTo = wrong[0].si;
    btn.on('pointerdown', () => {
      audioManager.playChoiceSelect();
      this.index = jumpTo;
      this._renderSentence();
    });
  }

  _clearSentenceArea() {
    this.sentenceEls.forEach(el => { if (el && el.destroy) { el.disableInteractive(); el.destroy(); } });
    this.sentenceEls = [];
  }

  destroy() {
    this.state = 'IDLE';
    this._clearOptions();
    this._clearSentenceArea();
    this.frameEls.forEach(el => { if (el && el.destroy) { el.disableInteractive(); el.destroy(); } });
    this.frameEls = [];
    this.elements = [];
    this.onComplete = null;
    this.scene.dialogueActive = false;
  }
}
