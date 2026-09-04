import Phaser from 'phaser';
import { audioManager } from '../audio/AudioManager.js';

const HEX = (c) => parseInt(c.replace('#', ''), 16);

/**
 * DialogueBox - themed Undertale-style dialogue.
 *
 * State machine: IDLE -> TYPING -> WAITING -> IDLE
 * Colors read from scene.uiAccent / scene.uiAccentDark so every biome gets
 * its own identity. destroy() fires onDestroy (used to clear dialogue lock).
 */
export class DialogueBox {
  constructor(scene) {
    this.scene = scene;
    this.state = 'IDLE';
    this.elements = [];
    this.optionEls = [];
    this.typewriterTimer = null;
    this.bodyText = null;
    this.fullText = '';
    this.options = null;
    this.onDestroyCb = null;
    this.overlay = null;
    this.accent = scene.uiAccent || '#ff5f9e';
    this.accentDark = scene.uiAccentDark || '#7a2a4a';
    this.accentNum = HEX(this.accent);
    this.accentDarkNum = HEX(this.accentDark);
  }

  show(speaker, text, portraitKey, expression, options, onDestroy) {
    if (this.state !== 'IDLE') return;
    // Accent may be set after construction, so read it per-show
    this.accent = this.scene.uiAccent || '#ff5f9e';
    this.accentDark = this.scene.uiAccentDark || '#7a2a4a';
    this.accentNum = HEX(this.accent);
    this.accentDarkNum = HEX(this.accentDark);

    this.state = 'TYPING';
    this.fullText = text;
    this.options = options || null;
    this.onDestroyCb = onDestroy || null;

    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;
    const boxH = 182;
    const boxY = h - boxH / 2 - 18;

    // ── Dim backdrop ──
    const dim = this.scene.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.58)
      .setDepth(90).setScrollFactor(0).setAlpha(0);
    this.scene.tweens.add({ targets: dim, alpha: 1, duration: 160 });
    this.elements.push(dim);

    // ── Portrait in a framed circle ──
    let portraitImg = null;
    if (portraitKey) {
      const px = 92;
      const py = boxY - 6;
      const frame = this.scene.add.graphics().setDepth(91).setScrollFactor(0);
      frame.fillStyle(this.accentDarkNum, 1);
      frame.fillCircle(px, py, 66);
      frame.fillStyle(0x1a0e24, 1);
      frame.fillCircle(px, py, 61);
      this.elements.push(frame);
      portraitImg = this.scene.add.image(px, py, portraitKey)
        .setDepth(92).setScrollFactor(0).setScale(1.5);
      if (expression && expression.tint) portraitImg.setTint(expression.tint);
      this.scene.tweens.add({
        targets: portraitImg, y: py - 4,
        duration: 1500, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
      });
      // subtle blink via quick scale dip
      this.scene.tweens.add({
        targets: portraitImg, scaleX: 1.42, scaleY: 1.42,
        duration: 90, yoyo: true, repeat: -1, repeatDelay: 2600,
        delay: 700 + Math.random() * 1400
      });
      this.elements.push(portraitImg);
    }

    // ── Box chrome ──
    const pad = portraitKey ? 172 : 46;
    const boxX = pad;
    const boxW = w - pad - 46;
    const gfx = this.scene.add.graphics().setDepth(91).setScrollFactor(0);
    // soft outer glow
    gfx.fillStyle(this.accentNum, 0.14);
    gfx.fillRoundedRect(boxX - 6, boxY - boxH / 2 - 6, boxW + 12, boxH + 12, 18);
    // main panel
    gfx.fillStyle(0x0a0812, 0.97);
    gfx.fillRoundedRect(boxX, boxY - boxH / 2, boxW, boxH, 12);
    // accent border
    gfx.lineStyle(2, this.accentNum, 0.85);
    gfx.strokeRoundedRect(boxX + 1, boxY - boxH / 2 + 1, boxW - 2, boxH - 2, 12);
    // inner hairline
    gfx.lineStyle(1, this.accentNum, 0.28);
    gfx.strokeRoundedRect(boxX + 7, boxY - boxH / 2 + 7, boxW - 14, boxH - 14, 9);
    // corner gems
    const gem = (gx, gy) => {
      gfx.fillStyle(this.accentNum, 1);
      gfx.fillTriangle(gx, gy - 6, gx + 6, gy, gx, gy + 6);
      gfx.fillTriangle(gx, gy - 6, gx - 6, gy, gx, gy + 6);
    };
    gem(boxX + 8, boxY - boxH / 2 + 8);
    gem(boxX + boxW - 8, boxY - boxH / 2 + 8);
    this.elements.push(gfx);

    // ── Speaker name plate ──
    const nameTag = this.scene.add.text(boxX + 20, boxY - boxH / 2 - 10, `\u2726 ${speaker} \u2726`, {
      fontFamily: '"Courier New", monospace',
      fontSize: '14px', fontStyle: 'bold',
      color: this.accent,
      backgroundColor: this.accentDark,
      padding: { x: 12, y: 5 }
    }).setDepth(93).setScrollFactor(0);
    this.elements.push(nameTag);

    // Divider
    const divider = this.scene.add.graphics().setDepth(93).setScrollFactor(0);
    divider.lineStyle(1, this.accentNum, 0.3);
    divider.lineBetween(boxX + 16, boxY - boxH / 2 + 24, boxX + boxW - 16, boxY - boxH / 2 + 24);
    this.elements.push(divider);

    // ── Body text ──
    this.bodyText = this.scene.add.text(boxX + 20, boxY - boxH / 2 + 32, '', {
      fontFamily: 'Georgia, "Times New Roman", serif',
      fontSize: '16px',
      color: '#e8e4e0',
      wordWrap: { width: boxW - 40, useAdvancedWrap: true },
      lineSpacing: 6
    }).setDepth(93).setScrollFactor(0);
    this.elements.push(this.bodyText);

    // ── Interactive overlay ──
    this.overlay = this.scene.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.001)
      .setDepth(100).setScrollFactor(0).setInteractive();
    this.elements.push(this.overlay);

    // ── Typewriter ──
    let ci = 0;
    this.typewriterTimer = this.scene.time.addEvent({
      delay: 22,
      callback: () => {
        if (this.state === 'IDLE') { this.typewriterTimer.remove(); return; }
        if (ci < this.fullText.length) {
          this.bodyText.text += this.fullText[ci];
          ci++;
          if (ci % 3 === 0) audioManager.playDialogueBlip();
        } else {
          this.typewriterTimer.remove();
          this.typewriterTimer = null;
          this._onTypewriterDone();
        }
      },
      repeat: this.fullText.length - 1
    });

    // ── Input ──
    this.overlay.on('pointerdown', () => this._handleInput());
    this._keyHandlers = [];
    const bindKey = (code, fn) => {
      this.scene.input.keyboard.on(code, fn);
      this._keyHandlers.push({ code, fn });
    };
    bindKey('keydown-SPACE', () => this._handleInput());
    bindKey('keydown-ENTER', () => this._handleInput());
  }

  _handleInput() {
    if (this.state === 'TYPING') {
      this.bodyText.text = this.fullText;
      if (this.typewriterTimer) { this.typewriterTimer.remove(); this.typewriterTimer = null; }
      this._onTypewriterDone();
    } else if (this.state === 'WAITING') {
      if (!this.options || this.options.length === 0) this.destroy();
    }
  }

  _onTypewriterDone() {
    this.state = 'WAITING';
    if (this.options && this.options.length > 0) this._showChoices();
    else this._showContinueArrow();
  }

  _showContinueArrow() {
    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;
    const ax = w - 58, ay = h - 36;
    const chip = this.scene.add.graphics().setDepth(94).setScrollFactor(0);
    chip.fillStyle(this.accentDarkNum, 1);
    chip.fillCircle(ax, ay, 16);
    chip.lineStyle(2, this.accentNum, 0.9);
    chip.strokeCircle(ax, ay, 16);
    this.elements.push(chip);
    const arrow = this.scene.add.text(ax, ay, '\u25BC', {
      fontSize: '13px', color: this.accent
    }).setOrigin(0.5).setDepth(94).setScrollFactor(0);
    this.scene.tweens.add({
      targets: [chip, arrow], y: ay + 4,
      duration: 380, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
    });
    this.elements.push(arrow);
  }

  _showChoices() {
    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;
    const boxH = 182;
    const boxY = h - boxH / 2 - 18;
    const startY = boxY - boxH / 2 - 12;

    if (this.overlay) this.overlay.disableInteractive();

    const count = this.options.length;
    const rowH = 46;
    const gap = 8;

    this.options.forEach((opt, i) => {
      const topY = startY - (i + 1) * (rowH + gap);
      const cy = topY + rowH / 2;

      const bg = this.scene.add.graphics().setDepth(101).setScrollFactor(0);
      bg.fillStyle(this.accentDarkNum, 0.22);
      bg.fillRoundedRect(w / 2 - 260, topY, 520, rowH, 12);
      bg.lineStyle(1, this.accentNum, 0.35);
      bg.strokeRoundedRect(w / 2 - 260, topY, 520, rowH, 12);
      // number chip
      bg.fillStyle(this.accentNum, 0.9);
      bg.fillCircle(w / 2 - 230, cy, 13);

      const numText = this.scene.add.text(w / 2 - 230, cy, `${i + 1}`, {
        fontFamily: '"Courier New", monospace', fontSize: '12px', fontStyle: 'bold', color: '#0a0812'
      }).setOrigin(0.5).setDepth(102).setScrollFactor(0);

      const label = this.scene.add.text(w / 2 - 200, cy, opt.text, {
        fontFamily: '"Courier New", monospace',
        fontSize: '15px',
        color: '#d8d4ee',
        wordWrap: { width: 430, useAdvancedWrap: true }
      }).setOrigin(0, 0.5).setDepth(102).setScrollFactor(0);

      const hitZone = this.scene.add.zone(w / 2, cy, 520, rowH)
        .setDepth(103).setScrollFactor(0).setInteractive({ useHandCursor: true });

      const onHover = () => {
        bg.clear();
        bg.fillStyle(this.accentNum, 0.16);
        bg.fillRoundedRect(w / 2 - 260, topY, 520, rowH, 12);
        bg.lineStyle(2, this.accentNum, 0.95);
        bg.strokeRoundedRect(w / 2 - 260, topY, 520, rowH, 12);
        bg.fillStyle(this.accentNum, 1);
        bg.fillCircle(w / 2 - 230, cy, 13);
        label.setColor('#ffffff');
      };
      const onOut = () => {
        bg.clear();
        bg.fillStyle(this.accentDarkNum, 0.22);
        bg.fillRoundedRect(w / 2 - 260, topY, 520, rowH, 12);
        bg.lineStyle(1, this.accentNum, 0.35);
        bg.strokeRoundedRect(w / 2 - 260, topY, 520, rowH, 12);
        bg.fillStyle(this.accentNum, 0.9);
        bg.fillCircle(w / 2 - 230, cy, 13);
        label.setColor('#d8d4ee');
      };

      hitZone.on('pointerover', onHover);
      hitZone.on('pointerout', onOut);
      hitZone.on('pointerdown', () => {
        audioManager.playChoiceSelect();
        this.destroy();
        if (opt.callback) opt.callback();
      });

      this.optionEls.push(bg, numText, label, hitZone);
    });

    // Number-key shortcuts (1-4)
    this.options.forEach((opt, i) => {
      if (i >= 4) return;
      const handler = () => {
        if (this.state !== 'WAITING') return;
        audioManager.playChoiceSelect();
        this.destroy();
        if (opt.callback) opt.callback();
      };
      this.scene.input.keyboard.once(`keydown-${i + 1}`, handler);
    });
  }

  destroy() {
    this.state = 'IDLE';
    if (this.typewriterTimer) {
      this.typewriterTimer.remove();
      this.typewriterTimer = null;
    }
    if (this._keyHandlers) {
      this._keyHandlers.forEach(({ code, fn }) => this.scene.input.keyboard.off(code, fn));
      this._keyHandlers = [];
    }
    const all = [...this.elements, ...this.optionEls];
    all.forEach(el => { if (el && el.destroy) { el.disableInteractive(); el.destroy(); } });
    this.elements = [];
    this.optionEls = [];
    this.options = null;
    this.bodyText = null;
    this.overlay = null;

    const cb = this.onDestroyCb;
    this.onDestroyCb = null;
    if (cb) cb();
  }
}
