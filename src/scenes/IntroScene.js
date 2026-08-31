import Phaser from 'phaser';
import { audioManager } from '../audio/AudioManager.js';
import { DialogueBox } from '../ui/DialogueBox.js';

/**
 * IntroScene - Undertale-style cutscene: pictures and text, not playable.
 * Nadia gets sick, Grandparent gives the book, River reads it, transported.
 */
export class IntroScene extends Phaser.Scene {
  constructor() {
    super({ key: 'IntroScene' });
  }

  create() {
    audioManager.init();
    audioManager.resume();
    this.cameras.main.setBackgroundColor('#0a0a0a');
    this.cameras.main.fadeIn(1000);
    this.dialogue = new DialogueBox(this);
    this._busy = false;

    this.showIntroPanels([
      'River and Nadia grew up in a small house with their older sibling Mark and their Grandparent.',
      'Life was hard. Mark worked various jobs, never finished school. River balanced work and school.',
      'Then one day, Nadia fell sick.',
      'The doctors called it "illiteracy" - a disease that made words blur and meaning slip away.',
      'River was desperate.'
    ], () => this.showGrandparent());
  }

  showIntroPanels(lines, onComplete) {
    this.children.removeAll();
    this.cameras.main.setBackgroundColor('#0a0a0a');
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;

    // Letterbox bars
    this.add.rectangle(w / 2, 0, w, 50, 0x000000).setOrigin(0.5, 0).setDepth(5);
    this.add.rectangle(w / 2, h, w, 50, 0x000000).setOrigin(0.5, 1).setDepth(5);

    let currentLine = 0;

    const showLine = () => {
      if (currentLine >= lines.length) { onComplete(); return; }
      if (this._busy) return;
      this._busy = true;

      const text = lines[currentLine];

      const txt = this.add.text(w / 2, h / 2, '', {
        fontFamily: 'Georgia, "Times New Roman", serif',
        fontSize: '20px',
        color: '#c8c0b0',
        wordWrap: { width: w - 180 },
        align: 'center',
        lineSpacing: 8
      }).setOrigin(0.5).setAlpha(0).setDepth(10);

      this.tweens.add({ targets: txt, alpha: 1, duration: 400 });

      let ci = 0;
      let typewriterDone = false;
      let advanced = false;

      const advance = () => {
        if (this._advanceTimer) { this._advanceTimer.remove(); this._advanceTimer = null; }
        this._busy = false;
        txt.destroy();
        currentLine++;
        showLine();
      };

      const skipToFull = () => {
        if (!typewriterDone) {
          txt.text = text;
          typewriterDone = true;
          this._advanceTimer = this.time.delayedCall(700, advance);
        }
      };

      // Single click handler - use once to avoid stacking
      const clickHandler = () => {
        if (this._busy) {
          if (!typewriterDone) skipToFull();
          else advance();
        }
      };
      this.input.once('pointerdown', clickHandler);

      // Also Space/Enter
      const spaceHandler = () => {
        if (this._busy) {
          if (!typewriterDone) skipToFull();
          else advance();
        }
      };
      this.input.keyboard.once('keydown-SPACE', spaceHandler);
      this.input.keyboard.once('keydown-ENTER', spaceHandler);

      const timer = this.time.addEvent({
        delay: 30,
        callback: () => {
          if (!this._busy || typewriterDone) { timer.remove(); return; }
          if (ci < text.length) { txt.text += text[ci]; ci++; }
          else {
            typewriterDone = true;
            timer.remove();
            this._advanceTimer = this.time.delayedCall(600, advance);
          }
        },
        repeat: text.length - 1
      });
    };

    showLine();
  }

  showGrandparent() {
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;
    this.children.removeAll();
    this._busy = false;

    this.cameras.main.setBackgroundColor('#1a1208');
    this.add.rectangle(w / 2, h / 2, w, h, 0x2a1a0a, 1).setDepth(0);

    // Grandparent portrait
    const g = this.add.graphics().setDepth(1);
    g.fillStyle(0xd2b48c, 1);
    g.fillEllipse(w / 2, h * 0.32, 130, 130);
    // Hair
    g.fillStyle(0x888888, 1);
    g.fillEllipse(w / 2, h * 0.20, 140, 60);
    // Eyes
    g.fillStyle(0xffffff, 1);
    g.fillCircle(w * 0.45, h * 0.30, 7);
    g.fillCircle(w * 0.55, h * 0.30, 7);
    g.fillStyle(0x333333, 1);
    g.fillCircle(w * 0.45, h * 0.30, 3.5);
    g.fillCircle(w * 0.55, h * 0.30, 3.5);
    // Smile
    g.lineStyle(2, 0x8a6a4a, 1);
    g.beginPath();
    g.arc(w / 2, h * 0.37, 14, 0.2, Math.PI - 0.2, false);
    g.strokePath();

    // Book glow
    const book = this.add.rectangle(w / 2, h * 0.58, 60, 45, 0x8B4513, 1).setDepth(2);
    this.tweens.add({ targets: book, alpha: { from: 0.7, to: 1 }, duration: 1000, yoyo: true, repeat: -1 });
    const glow = this.add.circle(w / 2, h * 0.58, 45, 0xffd700, 0.12).setDepth(1);
    this.tweens.add({ targets: glow, scaleX: 1.3, scaleY: 1.3, alpha: { from: 0.08, to: 0.18 }, duration: 1500, yoyo: true, repeat: -1 });

    this.time.delayedCall(500, () => {
      this.dialogue.show('Grandparent',
        'River, come here. I have something for you. This is our Family Heirloom. A book that might be the key to curing Nadia.',
        null, null,
        [{ text: 'Open the book', callback: () => this.readBook() }]
      );
    });
  }

  readBook() {
    audioManager.playScreenFlash();
    this.cameras.main.flash(1500, 255, 255, 200);
    this.cameras.main.shake(600, 0.02);

    this.time.delayedCall(2000, () => {
      this.cameras.main.fadeOut(1000, 255, 255, 255);
      this.cameras.main.once('camerafadeoutcomplete', () => {
        this.scene.start('CandyLandScene', {
          gameState: {
            hasHorse: false, completedRiddles: [], completedPuzzles: [],
            inventory: [], health: 100, lives: 3
          }
        });
      });
    });
  }
}
