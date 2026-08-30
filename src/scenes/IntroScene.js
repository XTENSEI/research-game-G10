import Phaser from 'phaser';
import { audioManager } from '../audio/AudioManager.js';
import { DialogueBox } from '../ui/DialogueBox.js';

/**
 * IntroScene - Undertale-style: pictures and text, not playable.
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
    this.step = 0;
    this.advanced = false;

    this.input.keyboard.on('keydown-SPACE', () => { this.advanced = true; });
    this.input.keyboard.on('keydown-ENTER', () => { this.advanced = true; });

    this.showPanel([
      { text: 'River and Nadia grew up in a small house with their older sibling Mark and their Grandparent.', delay: 500 },
      { text: 'Life was hard. Mark worked various jobs, never finished school. River balanced work and school.', delay: 3000 },
      { text: 'Then one day, Nadia fell sick.', delay: 5500 },
      { text: 'The doctors called it "illiteracy" - a disease that made words blur and meaning slip away.', delay: 7500 },
      { text: 'River was desperate.', delay: 10000 }
    ], () => this.showGrandparent());
  }

  showPanel(lines, onComplete) {
    this.children.removeAll();
    this.cameras.main.setBackgroundColor('#0a0a0a');

    // Cinematic letterbox bars
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;
    this.add.rectangle(w / 2, 0, w, 60, 0x000000).setOrigin(0.5, 0);
    this.add.rectangle(w / 2, h, w, 60, 0x000000).setOrigin(0.5, 1);

    let currentLine = 0;

    const showLine = () => {
      if (currentLine >= lines.length) { onComplete(); return; }
      const line = lines[currentLine];
      this.advanced = false;

      const txt = this.add.text(w / 2, h / 2, '', {
        fontFamily: '"Courier New", monospace',
        fontSize: '18px',
        color: '#cccccc',
        wordWrap: { width: w - 200 },
        align: 'center',
        lineSpacing: 8
      }).setOrigin(0.5).setAlpha(0);

      this.tweens.add({ targets: txt, alpha: 1, duration: 500 });

      // Typewriter
      let ci = 0;
      const timer = this.time.addEvent({
        delay: 30,
        callback: () => {
          if (this.advanced) {
            txt.text = line.text;
            timer.remove();
            this.time.delayedCall(800, () => { currentLine++; showLine(); });
            return;
          }
          if (ci < line.text.length) { txt.text += line.text[ci]; ci++; }
          else { timer.remove(); this.time.delayedCall(600, () => { currentLine++; showLine(); }); }
        },
        repeat: line.text.length - 1
      });
    };

    showLine();
  }

  showGrandparent() {
    // Portrait area
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;
    this.children.removeAll();
    this.cameras.main.setBackgroundColor('#1a1208');

    // Warm background
    this.add.rectangle(w / 2, h / 2, w, h, 0x2a1a0a, 1);

    // Grandparent portrait (simple, warm)
    const g = this.add.graphics();
    g.fillStyle(0xd2b48c, 1);
    g.fillEllipse(w / 2, h * 0.35, 120, 120);
    g.fillStyle(0x888888, 1);
    g.fillEllipse(w / 2, h * 0.22, 130, 60); // hair
    g.fillStyle(0xffffff, 1);
    g.fillCircle(w * 0.45, h * 0.33, 6);
    g.fillCircle(w * 0.55, h * 0.33, 6);
    g.fillStyle(0x333333, 1);
    g.fillCircle(w * 0.45, h * 0.33, 3);
    g.fillCircle(w * 0.55, h * 0.33, 3);
    g.lineStyle(2, 0x8a6a4a, 1);
    g.beginPath();
    g.arc(w / 2, h * 0.40, 12, 0.2, Math.PI - 0.2, false);
    g.strokePath();

    // Book glow
    const book = this.add.rectangle(w / 2, h * 0.6, 60, 45, 0x8B4513, 1);
    this.tweens.add({ targets: book, alpha: { from: 0.7, to: 1 }, duration: 1000, yoyo: true, repeat: -1 });
    const glow = this.add.circle(w / 2, h * 0.6, 40, 0xffd700, 0.15);
    this.tweens.add({ targets: glow, scaleX: 1.3, scaleY: 1.3, alpha: { from: 0.1, to: 0.2 }, duration: 1500, yoyo: true, repeat: -1 });

    this.time.delayedCall(500, () => {
      this.dialogue.show('Grandparent', 'River, come here. I have something for you. This is our Family Heirloom. A book that might be the key to curing Nadia.', [
        { text: 'Open the book', callback: () => this.readBook() }
      ]);
    });
  }

  readBook() {
    audioManager.playScreenFlash();
    this.cameras.main.flash(1500, 255, 255, 200);
    this.cameras.main.shake(600, 0.02);

    this.time.delayedCall(2000, () => {
      // Fade to white, then to Candy Land
      this.cameras.main.fadeOut(1000, 255, 255, 255);
      this.cameras.main.once('camerafadeoutcomplete', () => {
        this.scene.start('CandyLandScene', { gameState: { hasHorse: false, completedRiddles: [], completedPuzzles: [], inventory: [], health: 100, lives: 3 } });
      });
    });
  }
}
