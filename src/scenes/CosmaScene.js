import { BaseScene } from './BaseScene.js';

export class CosmaScene extends BaseScene {
  constructor() {
    super({ key: 'CosmaScene', biomeId: 'cosma' });
  }

  create(data) {
    super.create(data);

    this.cameras.main.fadeIn(1000);
    this.cameras.main.setBackgroundColor('#050510');

    // Parallax: deep space, neon stars, floating rocks
    this.setupParallax([
      { key: 'player', speed: 0.01, y: 0, depth: -3, tint: 0x0a0a2a },
      { key: 'player', speed: 0.08, y: 50, depth: -2, tint: 0xffdd00 },
      { key: 'player', speed: 0.2, y: 200, depth: -1, tint: 0x444466 },
      { key: 'player', speed: 0.5, y: 400, depth: 0, tint: 0x888899 }
    ]);

    // Player
    const player = this.createPlayer(100, 400);

    // Torches (3 podiums)
    this.torches = [];
    for (let i = 0; i < 3; i++) {
      const torch = this.physics.add.sprite(350 + i * 120, 380, 'torch-unlit');
      torch.setScale(3);
      torch.setImmovable(true);
      torch.setDepth(10);
      torch.lit = false;
      this.torches.push(torch);
    }

    // Stone slabs (3)
    this.slabsCollected = 0;

    // Particles: neon glow
    this.setupParticleEmitter('particle', {
      y: { min: 0, max: 576 },
      lifespan: 3000,
      speed: { min: 1, max: 5 },
      scale: { start: 0.5, end: 0 },
      alpha: { start: 0.8, end: 0 },
      frequency: 200,
      blendMode: 'ADD',
      tint: 0xffdd00,
      depth: 15
    });

    this.setupParticleEmitter('particle', {
      y: { min: 0, max: 576 },
      lifespan: 4000,
      speed: { min: 2, max: 8 },
      scale: { start: 0.3, end: 0 },
      alpha: { start: 0.6, end: 0 },
      frequency: 400,
      blendMode: 'ADD',
      tint: 0x44aaff,
      depth: 14
    });

    this.setupParticleEmitter('particle', {
      y: { min: 0, max: 576 },
      lifespan: 3500,
      speed: { min: 1, max: 4 },
      scale: { start: 0.4, end: 0 },
      alpha: { start: 0.7, end: 0 },
      frequency: 350,
      blendMode: 'ADD',
      tint: 0xff4444,
      depth: 13
    });

    // Rainbow Road intro
    this.time.delayedCall(800, () => {
      this.showDialogue('Narrator', 'The Rainbow Road stretches before you. Three unlit torches stand at the portal.', [
        {
          text: 'Begin star catching',
          callback: () => this.startStarCatching()
        }
      ]);
    });
  }

  startStarCatching() {
    // Spawn falling stars
    this.starTimer = this.time.addEvent({
      delay: 800,
      callback: () => this.spawnStar(),
      repeat: 19
    });
    this.starsCaught = 0;
    this.starsNeeded = 20;
    this.showDialogue('Narrator', 'Catch the golden stars! Avoid the red dying stars! (3 lives)', []);
  }

  spawnStar() {
    const x = Phaser.Math.Between(100, 900);
    const isDying = Math.random() < 0.3;
    const texture = isDying ? 'star-red' : 'star-gold';
    const star = this.physics.add.sprite(x, -20, texture);
    star.setScale(1.5);
    star.setDepth(11);
    star.isDying = isDying;
    star.setVelocityY(Phaser.Math.Between(80, 150));

    // Glow tween
    this.tweens.add({
      targets: star,
      alpha: { from: 0.6, to: 1 },
      scale: { from: 1.3, to: 1.7 },
      duration: 400,
      yoyo: true,
      repeat: -1
    });

    // Collect on overlap
    this.physics.add.overlap(this.player, star, () => {
      if (star.active) {
        star.destroy();
        if (isDying) {
          this.gameState.lives--;
          this.cameras.main.shake(200, 0.01);
          if (this.gameState.lives <= 0) {
            this.showDialogue('Narrator', 'No lives left! The stars fade away...', [
              { text: 'Retry', callback: () => this.startStarCatching() }
            ]);
          }
        } else {
          this.starsCaught++;
          // Flash gold
          this.cameras.main.flash(100, 255, 220, 0, false, null, this);
        }
        if (this.starsCaught >= this.starsNeeded) {
          this.starTimer.remove();
          this.time.delayedCall(500, () => this.placeSlabs());
        }
      }
    });
  }

  placeSlabs() {
    this.showDialogue('Narrator', 'The stars fuse into a poem carved on stone slabs. Place them on the torch podiums.', [
      {
        text: 'Place slabs',
        callback: () => {
          this.torches.forEach((torch, i) => {
            this.time.delayedCall(i * 500, () => {
              torch.setTexture('torch-lit');
              torch.lit = true;
              // Fire particle burst
              this.tweens.add({
                targets: torch,
                scaleY: 3.5,
                duration: 300,
                yoyo: true,
                ease: 'Bounce'
              });
            });
          });
          this.time.delayedCall(2000, () => this.summonGenie());
        }
      }
    ]);
  }

  summonGenie() {
    this.showDialogue('Magic Fire Genie', 'I am the Fire Genie! You have lit my torches. Now answer this: What is the meaning of the poem?', [
      {
        text: 'Impact outlasts presence',
        callback: () => {
          this.showDialogue('Fire Genie', 'Correct! The poem speaks of how people shape us even after they\'re gone.', [
            { text: 'Continue', callback: () => this.transitionToEnding() }
          ]);
        }
      },
      {
        text: 'Stars are pretty',
        callback: () => {
          this.showDialogue('Fire Genie', 'Wrong! The poem restarts with new verses...', [
            { text: 'Retry', callback: () => this.startStarCatching() }
          ]);
        }
      }
    ]);
  }

  transitionToEnding() {
    this.cameras.main.fadeOut(1500, 255, 255, 255);
    this.cameras.main.once('camerafadeoutcomplete', () => {
      this.scene.start('EndingScene', { gameState: this.gameState });
    });
  }

  update() {
    this.handleMovement(180);
    this.updateParallax();
  }
}
