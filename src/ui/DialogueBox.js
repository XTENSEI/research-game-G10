import Phaser from 'phaser';
import { audioManager } from '../audio/AudioManager.js';

/**
 * DialogueBox - Undertale-style dialogue with character portraits,
 * expressions, typewriter text, and choice selection.
 */
export class DialogueBox {
  constructor(scene) {
    this.scene = scene;
    this.elements = [];
    this.active = false;
    this.typewriterTimer = null;
    this.skipRequested = false;
  }

  show(speaker, text, portrait, expression, options = null) {
    if (this.active) return;
    this.active = true;
    this.skipRequested = false;

    const w = this.scene.cameras.main.width;
    const h = this.scene.cameras.main.height;
    const boxH = 180;
    const boxY = h - boxH / 2 - 20;

    // ── Dim overlay ──
    const overlay = this.scene.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.5)
      .setDepth(90).setScrollFactor(0).setAlpha(0);
    this.scene.tweens.add({ targets: overlay, alpha: 1, duration: 200 });

    // ── Portrait (left side) ──
    const portraitX = 120;
    const portraitY = boxY;
    let portraitImg = null;
    if (portrait) {
      portraitImg = this.scene.add.image(portraitX, portraitY, portrait)
        .setDepth(92).setScrollFactor(0).setScale(1.5);
      // Expression change
      if (expression) {
        portraitImg.setTint(expression.tint || 0xffffff);
      }
      // Gentle float
      this.scene.tweens.add({
        targets: portraitImg, y: portraitY - 4,
        duration: 1500, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
      });
    }

    // ── Dialogue box (rounded, styled) ──
    const boxX = portrait ? 220 : 40;
    const boxW = portrait ? w - 260 : w - 80;
    const box = this.scene.add.graphics().setDepth(91).setScrollFactor(0);

    // Outer glow
    box.fillStyle(0xff69b4, 0.15);
    box.fillRoundedRect(boxX - 4, boxY - boxH / 2 - 4, boxW + 8, boxH + 8, 16);
    // Main box
    box.fillStyle(0x0d0d1a, 0.95);
    box.fillRoundedRect(boxX, boxY - boxH / 2, boxW, boxH, 12);
    // Border
    box.lineStyle(2, 0xff69b4, 0.8);
    box.strokeRoundedRect(boxX, boxY - boxH / 2, boxW, boxH, 12);
    // Inner accent line
    box.lineStyle(1, 0xff69b4, 0.2);
    box.strokeRoundedRect(boxX + 6, boxY - boxH / 2 + 6, boxW - 12, boxH - 12, 8);

    // ── Speaker name ──
    const nameTag = this.scene.add.text(boxX + 20, boxY - boxH / 2 + 10, speaker, {
      fontFamily: '"Courier New", monospace',
      fontSize: '16px',
      fontStyle: 'bold',
      color: '#ff69b4',
      stroke: '#000000',
      strokeThickness: 3
    }).setDepth(93).setScrollFactor(0);

    // ── Divider line under name ──
    const divider = this.scene.add.graphics().setDepth(93).setScrollFactor(0);
    divider.lineStyle(1, 0xff69b4, 0.3);
    divider.lineBetween(boxX + 20, boxY - boxH / 2 + 32, boxX + boxW - 20, boxY - boxH / 2 + 32);

    // ── Typewriter text ──
    const bodyText = this.scene.add.text(boxX + 20, boxY - boxH / 2 + 40, '', {
      fontFamily: '"Courier New", monospace',
      fontSize: '15px',
      color: '#e0e0e0',
      wordWrap: { width: boxW - 40, useAdvancedWrap: true },
      lineSpacing: 6
    }).setDepth(93).setScrollFactor(0);

    this.elements = [overlay, box, nameTag, divider, bodyText];
    if (portraitImg) this.elements.push(portraitImg);

    // ── Typewriter effect ──
    let charIndex = 0;
    const typeTimer = this.scene.time.addEvent({
      delay: 28,
      callback: () => {
        if (this.skipRequested) {
          bodyText.text = text;
          typeTimer.remove();
          this._showPrompt(options, w, h, boxX, boxY, boxW, boxH);
          return;
        }
        if (charIndex < text.length) {
          bodyText.text += text[charIndex];
          charIndex++;
          // Blip sound every 2 chars
          if (charIndex % 2 === 0) audioManager.playDialogueBlip();
        } else {
          typeTimer.remove();
          this._showPrompt(options, w, h, boxX, boxY, boxW, boxH);
        }
      },
      repeat: text.length - 1
    });
    this.typewriterTimer = typeTimer;

    // Skip with Space or click
    const skipHandler = () => {
      if (this.typewriterTimer && this.typewriterTimer.getRepeatCount() > 0) {
        this.skipRequested = true;
      }
    };
    this.scene.input.keyboard.once('keydown-SPACE', skipHandler);
    this.scene.input.keyboard.once('keydown-ENTER', skipHandler);
  }

  _showPrompt(options, w, h, boxX, boxY, boxW, boxH) {
    if (options && options.length > 0) {
      this._showChoices(options, w, h, boxX, boxY, boxW, boxH);
    } else {
      this._showContinueArrow(w, h, boxX, boxY, boxW, boxH);
    }
  }

  _showContinueArrow(w, h, boxX, boxY, boxW, boxH) {
    const arrow = this.scene.add.text(boxX + boxW - 30, boxY + boxH / 2 - 25, '▼', {
      fontSize: '16px', color: '#ff69b4'
    }).setOrigin(0.5).setDepth(94).setScrollFactor(0);

    this.scene.tweens.add({
      targets: arrow, y: arrow.y + 5,
      duration: 400, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
    });
    this.elements.push(arrow);

    const advance = () => {
      this.scene.input.keyboard.off('keydown-SPACE', advance);
      this.scene.input.keyboard.off('keydown-ENTER', advance);
      audioManager.playChoiceSelect();
      this.destroy();
    };
    this.scene.input.keyboard.once('keydown-SPACE', advance);
    this.scene.input.keyboard.once('keydown-ENTER', advance);
  }

  _showChoices(options, w, h, boxX, boxY, boxW, boxH) {
    const choiceY = boxY - boxH / 2 - 10;
    const choiceBtns = [];

    options.forEach((opt, i) => {
      const by = choiceY - (options.length - i) * 46 - 10;

      const bg = this.scene.add.graphics().setDepth(95).setScrollFactor(0);
      bg.fillStyle(0x1a1a2e, 0.95);
      bg.fillRoundedRect(w / 2 - 200, by, 400, 40, 8);
      bg.lineStyle(1, 0x6666aa, 0.6);
      bg.strokeRoundedRect(w / 2 - 200, by, 400, 40, 8);

      const label = this.scene.add.text(w / 2, by + 20, `> ${opt.text}`, {
        fontFamily: '"Courier New", monospace',
        fontSize: '15px',
        color: '#aaaaaa'
      }).setOrigin(0.5).setDepth(96).setScrollFactor(0);

      const hitZone = this.scene.add.zone(w / 2, by + 20, 400, 40)
        .setDepth(97).setScrollFactor(0).setInteractive({ useHandCursor: true });

      hitZone.on('pointerover', () => {
        bg.clear();
        bg.fillStyle(0x2a1a3e, 0.98);
        bg.fillRoundedRect(w / 2 - 200, by, 400, 40, 8);
        bg.lineStyle(2, 0xff69b4, 1);
        bg.strokeRoundedRect(w / 2 - 200, by, 400, 40, 8);
        label.setColor('#ff69b4');
        label.setText(opt.text);
      });

      hitZone.on('pointerout', () => {
        bg.clear();
        bg.fillStyle(0x1a1a2e, 0.95);
        bg.fillRoundedRect(w / 2 - 200, by, 400, 40, 8);
        bg.lineStyle(1, 0x6666aa, 0.6);
        bg.strokeRoundedRect(w / 2 - 200, by, 400, 40, 8);
        label.setColor('#aaaaaa');
        label.setText(`> ${opt.text}`);
      });

      hitZone.on('pointerdown', () => {
        audioManager.playChoiceSelect();
        this.destroy();
        if (opt.callback) opt.callback();
      });

      choiceBtns.push({ bg, label, hitZone });
      this.elements.push(bg, label, hitZone);
    });

    // Also allow number keys
    options.forEach((opt, i) => {
      const key = this.scene.input.keyboard.addKey(
        Phaser.Input.Keyboard.KeyCodes[`${i + 1}`] || (49 + i)
      );
      key.once('down', () => {
        audioManager.playChoiceSelect();
        this.destroy();
        if (opt.callback) opt.callback();
      });
    });
  }

  destroy() {
    this.active = false;
    if (this.typewriterTimer) {
      this.typewriterTimer.remove();
      this.typewriterTimer = null;
    }
    this.elements.forEach(el => {
      if (el && el.destroy) el.destroy();
    });
    this.elements = [];
  }
}
