import { BaseScene } from './BaseScene.js';
import { audioManager } from '../audio/AudioManager.js';
import { RiddleBox } from '../ui/RiddleBox.js';
import cosmaRiddleData from '../../riddles/cosma-riddles.json';
import rainbowData from '../../puzzles/cosma-rainbow-road.json';

const COSMA_RIDDLES = cosmaRiddleData.riddles;
const POEMS = rainbowData.poems;
const STARS_NEEDED = rainbowData.stars_needed || 15;
const MAX_LIVES = rainbowData.lives || 3;

export class CosmaScene extends BaseScene {
  constructor() {
    super({ key: 'CosmaScene', biomeId: 'cosma' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1500);
    this.cameras.main.setBackgroundColor('#020208');

    this.setupGround(0x0a0a1e, 450);

    // Deep space parallax
    this.setupParallax([
      { key: 'particle', speed: 0.005, y: 0, depth: -5, tint: 0x0a0a2a, alpha: 0.5 },
      { key: 'particle', speed: 0.03, y: 60, depth: -4, tint: 0x222244, alpha: 0.4 },
      { key: 'sparkle-gold', speed: 0.08, y: 150, depth: -3, tint: 0xffdd00, alpha: 0.3 },
      { key: 'particle', speed: 0.15, y: 300, depth: -2, tint: 0x444466, alpha: 0.3 },
      { key: 'particle', speed: 0.35, y: 430, depth: -1, tint: 0x666688, alpha: 0.2 }
    ]);

    const player = this.createPlayer(100, 400);

    this.riddleBox = new RiddleBox(this);
    this.riddleQueue = [];

    // Neon star particles (multi-colored)
    this.setupEmitter('sparkle-gold', {
      y: { min: 0, max: 576 },
      lifespan: 3500,
      speed: { min: 1, max: 5 },
      scale: { start: 0.4, end: 0 },
      alpha: { start: 0.7, end: 0 },
      frequency: 180,
      blendMode: 'ADD',
      tint: [0xffdd00, 0x44aaff, 0xff4444, 0xffffff],
      depth: 13
    });

    // Floating dust
    this.setupEmitter('particle', {
      y: { min: 0, max: 576 },
      lifespan: 5000,
      speed: { min: 1, max: 3 },
      scale: { start: 0.2, end: 0 },
      alpha: { start: 0.3, end: 0 },
      frequency: 400,
      blendMode: 'ADD',
      tint: 0x8888aa,
      depth: 1
    });

    this.setupVignette(0.6, 89);

    audioManager.playAmbientCosma();

    // Torches
    this.torches = [];
    for (let i = 0; i < 3; i++) {
      const torch = this.physics.add.sprite(380 + i * 130, 380, 'torch-unlit');
      torch.setScale(3);
      torch.setImmovable(true);
      torch.setDepth(10);
      torch.lit = false;
      this.torches.push(torch);
    }

    this.starsCaught = 0;
    this.lives = MAX_LIVES;
    this.starActive = false;
    this.currentPoem = null;

    this.time.delayedCall(800, () => {
      this.showDialogue('Narrator',
        'The Rainbow Road stretches before you. Three unlit torches stand at the portal. But first, the void tests your understanding.',
        [{ text: 'Listen to the void', callback: () => this.cosmaRiddles() }],
        'portrait-river'
      );
    });
  }

  // ─── Riddles (enforce >=1 before puzzle) ────────────────

  cosmaRiddles() {
    const done = this.gameState.completedRiddles || [];
    this.riddleQueue = COSMA_RIDDLES.filter(r => !done.includes(r.id));
    this.askNextRiddle();
  }

  askNextRiddle() {
    if (this.riddleQueue.length === 0) { this.rainbowIntro(); return; }
    const r = this.riddleQueue.shift();
    this.riddleBox.show({
      title: 'Cosmic Riddle',
      question: r.question,
      options: r.options,
      correct: r.correct,
      explanation: r.explanation,
      onComplete: (ok) => {
        const done = this.gameState.completedRiddles || (this.gameState.completedRiddles = []);
        if (!done.includes(r.id)) done.push(r.id);
        this.time.delayedCall(300, () => {
          if (this.riddleQueue.length > 0) {
            this.showDialogue('Void', 'One more whisper, or shall the road ignite?', [
              { text: 'Hear another', callback: () => this.askNextRiddle() },
              { text: 'Light the road', callback: () => this.rainbowIntro() }
            ]);
          } else {
            this.showDialogue('Void', 'Your mind is bright. Let the road burn.', [
              { text: 'Begin', callback: () => this.rainbowIntro() }
            ]);
          }
        });
      }
    });
  }

  // ─── Star Catching ──────────────────────────────────────

  rainbowIntro() {
    this.showDialogue('Narrator',
      'Catch the falling stars to fill the meter. Avoid the dying red stars - they drain the meter and cost a life. Three lives, traveler.',
      [{ text: 'Begin', callback: () => this.startCatching() }],
      'portrait-river'
    );
  }

  startCatching() {
    this.starsCaught = 0;
    this.lives = MAX_LIVES;
    this.starActive = true;
    this.currentPoem = this._pickRandomPoem(this.currentPoem);
    this._buildHud();

    this.starTimer = this.time.addEvent({
      delay: 650,
      callback: () => this.spawnStar(),
      loop: true
    });
  }

  _pickRandomPoem(previous) {
    const pool = POEMS.filter(p => p.id !== (previous && previous.id));
    return pool[Math.floor(Math.random() * pool.length)];
  }

  _buildHud() {
    if (this.hudGroup) this.hudGroup.destroy(true);
    this.hudGroup = this.add.container(0, 0).setDepth(95).setScrollFactor(0);
    const w = this.cameras.main.width;

    this.hudStars = this.add.text(24, 20, `\u2605 ${this.starsCaught}/${STARS_NEEDED}`, {
      fontFamily: '"Courier New", monospace', fontSize: '18px', color: '#ffdd00',
      stroke: '#000000', strokeThickness: 3
    }).setDepth(95).setScrollFactor(0);

    let hearts = '';
    for (let i = 0; i < MAX_LIVES; i++) hearts += i < this.lives ? '\u2665' : '\u2661';
    this.hudLives = this.add.text(w - 24, 20, hearts, {
      fontFamily: '"Courier New", monospace', fontSize: '20px', color: '#ff5555',
      stroke: '#000000', strokeThickness: 3
    }).setOrigin(1, 0).setDepth(95).setScrollFactor(0);

    // Meter bar
    const barW = 300, barH = 14;
    this.meterBg = this.add.rectangle(w / 2, 24, barW, barH, 0x111133, 0.9)
      .setStrokeStyle(1, 0x6666aa, 0.8).setDepth(95).setScrollFactor(0);
    this.meterFill = this.add.rectangle(w / 2 - barW / 2, 24, 0, barH - 4, 0xffdd00, 0.9)
      .setOrigin(0, 0.5).setDepth(96).setScrollFactor(0);

    this.hudGroup.add([this.hudStars, this.hudLives, this.meterBg, this.meterFill]);
  }

  _updateHud() {
    if (this.hudStars) this.hudStars.setText(`\u2605 ${this.starsCaught}/${STARS_NEEDED}`);
    if (this.hudLives) {
      let hearts = '';
      for (let i = 0; i < MAX_LIVES; i++) hearts += i < this.lives ? '\u2665' : '\u2661';
      this.hudLives.setText(hearts);
    }
    if (this.meterFill) {
      this.meterFill.width = Math.max(0, (296) * (this.starsCaught / STARS_NEEDED));
    }
  }

  spawnStar() {
    if (!this.starActive) return;
    const x = Phaser.Math.Between(80, 940);
    const isDying = Math.random() < 0.25;
    const tex = isDying ? 'particle' : 'sparkle-gold';
    const tint = isDying ? 0xff2222 : 0xffdd00;

    const star = this.physics.add.sprite(x, -15, tex);
    star.setScale(isDying ? 0.8 : 1.2);
    star.setDepth(11);
    star.isDying = isDying;
    star.setVelocityY(Phaser.Math.Between(100, 180));
    star.setTint(tint);

    this.tweens.add({
      targets: star, alpha: { from: 0.6, to: 1 }, scale: star.scale * 1.3,
      duration: 300, yoyo: true, repeat: -1
    });

    this.physics.add.overlap(this.player, star, () => {
      if (!star.active) return;
      star.destroy();
      if (isDying) {
        this.lives--;
        audioManager.playWrong();
        this.cameras.main.shake(150, 0.012);
        this.cameras.main.flash(150, 255, 50, 50, false);
        this._updateHud();
        if (this.lives <= 0) this.failCatching();
      } else {
        this.starsCaught++;
        audioManager.playCorrect();
        this.cameras.main.flash(80, 255, 220, 0, false);
        this._updateHud();
        if (this.starsCaught >= STARS_NEEDED) this.completeCatching();
      }
    });
  }

  failCatching() {
    this.starActive = false;
    if (this.starTimer) this.starTimer.remove();
    if (this.hudGroup) this.hudGroup.destroy(true);
    this.showDialogue('Narrator',
      'No lives left! The stars fade away, and the poem is lost...',
      [{ text: 'Retry', callback: () => this.startCatching() }],
      'portrait-river'
    );
  }

  completeCatching() {
    this.starActive = false;
    if (this.starTimer) this.starTimer.remove();
    if (this.hudGroup) this.hudGroup.destroy(true);
    audioManager.playScreenFlash();
    this.cameras.main.flash(500, 255, 220, 0);

    this.time.delayedCall(600, () => {
      const poem = this.currentPoem;
      this.showDialogue('Narrator',
        `The stars fuse into a poem called \u201C${poem.title}\u201D, carved across three stone slabs:\n\n${poem.fragments.map(f => `\u2022 ${f}`).join('\n')}`,
        [{ text: 'Place the slabs', callback: () => this.lightTorches() }],
        'portrait-river'
      );
    });
  }

  lightTorches() {
    this.torches.forEach((torch, i) => {
      this.time.delayedCall(i * 600, () => {
        torch.setTexture('torch-lit');
        torch.lit = true;
        audioManager.playScreenFlash();
        this.cameras.main.flash(150, 255, 150, 0, false);
        this.tweens.add({
          targets: torch, scaleY: 3.8, duration: 200, yoyo: true, ease: 'Bounce'
        });
      });
    });

    this.time.delayedCall(2200, () => this.summonGenie());
  }

  summonGenie() {
    audioManager.playScreenFlash();
    this.cameras.main.flash(800, 255, 100, 0);

    this.time.delayedCall(600, () => {
      const poem = this.currentPoem;
      this.showDialogue('Magic Fire Genie',
        `I am the Fire Genie! You have lit my torches. Now answer this: what is the meaning of the poem \u201C${poem.title}\u201D?`,
        [
          { text: poem.options[0], callback: () => this.genieCorrect() },
          { text: poem.options[1], callback: () => this.genieWrong() },
          { text: poem.options[2], callback: () => this.genieWrong() },
          { text: poem.options[3], callback: () => this.genieWrong() }
        ],
        'portrait-genie'
      );
    });
  }

  genieCorrect() {
    const poem = this.currentPoem;
    const puzzles = this.gameState.completedPuzzles || (this.gameState.completedPuzzles = []);
    if (!puzzles.includes('rainbow-road')) puzzles.push('rainbow-road');
    const inv = this.gameState.inventory || (this.gameState.inventory = []);
    if (!inv.includes('cosma-ingredient')) inv.push('cosma-ingredient');

    audioManager.playCorrect();
    this.cameras.main.flash(500, 255, 220, 100);
    this.showDialogue('Fire Genie',
      `Correct! \u201C${poem.title}\u201D \u2014 ${poem.meaning} You have earned the final essence: the Gift of Education itself.`,
      [{ text: 'Take the gift', callback: () => this.transitionTo('EndingScene', 2000, 0xffffff) }],
      'portrait-genie'
    );
  }

  genieWrong() {
    audioManager.playWrong();
    this.cameras.main.shake(300, 0.015);
    this.showDialogue('Fire Genie',
      'Wrong! The poem scatters into stardust and re-forms as something new...',
      [{ text: 'Retry', callback: () => this.startCatching() }],
      'portrait-genie'
    );
  }

  update() {
    this.handleMovement(180);
    this.updateParallax();
  }
}
