import Phaser from 'phaser';
import { audioManager } from '../audio/AudioManager.js';

/**
 * DialogueBox - Clean Undertale-style dialogue system.
 *
 * State machine: IDLE -> TYPING -> WAITING -> IDLE
 * - IDLE: nothing showing
 * - TYPING: typewriter effect running, click/Space skips to end
 * - WAITING: full text shown, waiting for click/Space to advance (continue) or choice selection
 *
 * No global scene click handlers. All input goes through a dedicated interactive
 * overlay zone that sits above the dialogue box and consumes clicks cleanly.
 */
export class DialogueBox {
  constructor(scene) {
    this.scene = scene;
    this.state = 'IDLE'; // IDLE | TYPING | WAITING
    this.elements = [];
    this.typewriterTimer = null;
    this.bodyText = null;
    this.fullText = '';
    this.options = null;
    this.onDestroyCb = null;

    // Single interactive overlay for all input (prevents double-fire)
    this.overlay = null;
  }

  /**
   * Show a dialogue box.
   * @param {string} speaker
   * @param {string} text
   * @param {string|null} portraitKey - texture key for portrait image
   * @param {object|null} expression
   * @param {Array|null} options - [{text, callback}] for choices, null for continue
   * @param {function|null} onDestroy - called when box is destroyed (sync, before next dialogue)
   */
  show(speaker, text, portraitKey, expression, options, onDestroy) {
    if (this.state !== 'IDLE') return;
    this.state = 'TYPING';
    this.fullText = text;
    this.options = options || null;
    this.onDestroyCb = onDestroy || null;

    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;
    const boxH = 180;
    const boxY = h - boxH / 2 - 20;

    // ── Dim backdrop ──
    const dim = this.scene.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.6)
      .setDepth(90).setScrollFactor(0).setAlpha(0);
    this.scene.tweens.add({ targets: dim, alpha: 1, duration: 150 });
    this.elements.push(dim);

    // ── Portrait ──
    let portraitImg = null;
    if (portraitKey) {
      const px = 120;
      portraitImg = this.scene.add.image(px, boxY, portraitKey)
        .setDepth(92).setScrollFactor(0).setScale(1.5);
      if (expression && expression.tint) portraitImg.setTint(expression.tint);
      this.scene.tweens.add({
        targets: portraitImg, y: boxY - 4,
        duration: 1500, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
      });
      this.elements.push(portraitImg);
    }

    // ── Box ──
    const pad = portraitKey ? 220 : 40;
    const boxX = pad;
    const boxW = w - pad - 40;

    const boxGfx = this.scene.add.graphics().setDepth(91).setScrollFactor(0);
    // Outer glow
    boxGfx.fillStyle(0xff69b4, 0.12);
    boxGfx.fillRoundedRect(boxX - 3, boxY - boxH / 2 - 3, boxW + 6, boxH + 6, 14);
    // Main bg
    boxGfx.fillStyle(0x080812, 0.96);
    boxGfx.fillRoundedRect(boxX, boxY - boxH / 2, boxW, boxH, 10);
    // Border
    boxGfx.lineStyle(2, 0xff69b4, 0.7);
    boxGfx.strokeRoundedRect(boxX, boxY - boxH / 2, boxW, boxH, 10);
    this.elements.push(boxGfx);

    // ── Speaker name ──
    const nameTag = this.scene.add.text(boxX + 16, boxY - boxH / 2 + 8, speaker, {
      fontFamily: 'Arial, Helvetica, sans-serif',
      fontSize: '15px',
      fontStyle: 'bold',
      color: '#ff69b4',
      stroke: '#000000',
      strokeThickness: 3
    }).setDepth(93).setScrollFactor(0);
    this.elements.push(nameTag);

    // ── Divider ──
    const divider = this.scene.add.graphics().setDepth(93).setScrollFactor(0);
    divider.lineStyle(1, 0xff69b4, 0.25);
    divider.lineBetween(boxX + 16, boxY - boxH / 2 + 30, boxX + boxW - 16, boxY - boxH / 2 + 30);
    this.elements.push(divider);

    // ── Body text ──
    this.bodyText = this.scene.add.text(boxX + 16, boxY - boxH / 2 + 38, '', {
      fontFamily: 'Arial, Helvetica, sans-serif',
      fontSize: '15px',
      color: '#d8d8d8',
      wordWrap: { width: boxW - 32, useAdvancedWrap: true },
      lineSpacing: 5
    }).setDepth(93).setScrollFactor(0);
    this.elements.push(this.bodyText);

    // ── Interactive overlay (consumes all clicks while dialogue is open) ──
    this.overlay = this.scene.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.001)
      .setDepth(100).setScrollFactor(0).setInteractive();
    this.elements.push(this.overlay);

    // ── Typewriter ──
    let ci = 0;
    this.typewriterTimer = this.scene.time.addEvent({
      delay: 25,
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

    // ── Input: overlay click ──
    this.overlay.on('pointerdown', () => this._handleInput());

    // ── Input: Space / Enter ──
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
      // Skip typewriter, show full text
      this.bodyText.text = this.fullText;
      if (this.typewriterTimer) {
        this.typewriterTimer.remove();
        this.typewriterTimer = null;
      }
      this._onTypewriterDone();
    } else if (this.state === 'WAITING') {
      if (!this.options || this.options.length === 0) {
        // Continue dialog
        this.destroy();
      }
      // If there are choices, ignore click (must click a specific choice)
    }
  }

  _onTypewriterDone() {
    this.state = 'WAITING';

    if (this.options && this.options.length > 0) {
      this._showChoices();
    } else {
      this._showContinueArrow();
    }
  }

  _showContinueArrow() {
    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;
    const arrow = this.scene.add.text(w - 50, h - 45, '\u25BC', {
      fontSize: '16px', color: '#ff69b4'
    }).setOrigin(0.5).setDepth(94).setScrollFactor(0);
    this.scene.tweens.add({
      targets: arrow, y: arrow.y + 5,
      duration: 350, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
    });
    this.elements.push(arrow);
    // Overlay handles click-to-advance for continue dialogs
    // (already set up in show())
  }

  _showChoices() {
    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;
    const boxH = 180;
    const boxY = h - boxH / 2 - 20;
    const startY = boxY - boxH / 2 - 10;

    // Disable overlay so clicks reach the choice hitZones below it
    if (this.overlay) this.overlay.disableInteractive();

    this.options.forEach((opt, i) => {
      const by = startY - (this.options.length - i) * 48 - 8;

      const bg = this.scene.add.graphics().setDepth(101).setScrollFactor(0);
      bg.fillStyle(0x111122, 0.96);
      bg.fillRoundedRect(w / 2 - 220, by, 440, 42, 8);
      bg.lineStyle(1, 0x555588, 0.5);
      bg.strokeRoundedRect(w / 2 - 220, by, 440, 42, 8);

      const label = this.scene.add.text(w / 2, by + 21, `\u203A ${opt.text}`, {
        fontFamily: 'Arial, Helvetica, sans-serif',
        fontSize: '15px',
        color: '#999999'
      }).setOrigin(0.5).setDepth(102).setScrollFactor(0);

      const hitZone = this.scene.add.zone(w / 2, by + 21, 440, 42)
        .setDepth(103).setScrollFactor(0).setInteractive({ useHandCursor: true });

      const onHover = () => {
        bg.clear();
        bg.fillStyle(0x1a1a3a, 0.98);
        bg.fillRoundedRect(w / 2 - 220, by, 440, 42, 8);
        bg.lineStyle(2, 0xff69b4, 1);
        bg.strokeRoundedRect(w / 2 - 220, by, 440, 42, 8);
        label.setColor('#ff69b4');
        label.setText(opt.text);
      };
      const onOut = () => {
        bg.clear();
        bg.fillStyle(0x111122, 0.96);
        bg.fillRoundedRect(w / 2 - 220, by, 440, 42, 8);
        bg.lineStyle(1, 0x555588, 0.5);
        bg.strokeRoundedRect(w / 2 - 220, by, 440, 42, 8);
        label.setColor('#999999');
        label.setText(`\u203A ${opt.text}`);
      };

      hitZone.on('pointerover', onHover);
      hitZone.on('pointerout', onOut);
      hitZone.on('pointerdown', () => {
        audioManager.playChoiceSelect();
        this.destroy();
        if (opt.callback) opt.callback();
      });

      this.elements.push(bg, label, hitZone);
    });

    // Number key shortcuts (1-4)
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
    // Remove keyboard handlers
    if (this._keyHandlers) {
      this._keyHandlers.forEach(({ code, fn }) => {
        this.scene.input.keyboard.off(code, fn);
      });
      this._keyHandlers = [];
    }
    // Destroy all visual elements
    this.elements.forEach(el => {
      if (el && el.destroy) {
        el.disableInteractive();
        el.destroy();
      }
    });
    this.elements = [];
    this.options = null;
    this.bodyText = null;
    this.overlay = null;

    // Fire onDestroy (BaseScene uses it to clear dialogueActive)
    const cb = this.onDestroyCb;
    this.onDestroyCb = null;
    if (cb) cb();
  }
}
