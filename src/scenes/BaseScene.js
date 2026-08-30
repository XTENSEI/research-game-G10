import Phaser from 'phaser';

/**
 * Base scene with shared functionality:
 * - WASD movement
 * - Parallax backgrounds
 * - Particle effects
 * - Dialogue system (Undertale-style)
 * - Tween animations
 */
export class BaseScene extends Phaser.Scene {
  constructor(key, biomeId) {
    super({ key });
    this.biomeId = biomeId;
    this.player = null;
    this.cursors = null;
    this.wasd = null;
    this.npcGroup = null;
    this.dialogueActive = false;
    this.parallaxLayers = [];
    this.particleEmitters = [];
  }

  init(data) {
    this.gameState = data.gameState || {
      hasHorse: false,
      completedRiddles: [],
      completedPuzzles: [],
      inventory: [],
      health: 100,
      lives: 3
    };
  }

  create(data) {
    // WASD + Arrow keys
    this.cursors = this.input.keyboard.createCursorKeys();
    this.wasd = {
      W: this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.W),
      A: this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.A),
      S: this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.S),
      D: this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.D)
    };

    this.npcGroup = this.physics.add.group();

    // Subclass must call this.setupWorld()
  }

  // ─── Parallax ──────────────────────────────────────────
  setupParallax(layers) {
    // layers: [{key, speed, y, tint}]
    this.parallaxLayers = [];
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;

    layers.forEach((layer) => {
      const bg = this.add.tileSprite(0, layer.y || h / 2, w * 2, h, layer.key)
        .setOrigin(0.5)
        .setScrollFactor(0)
        .setDepth(layer.depth || 0);
      if (layer.tint) bg.setTint(layer.tint);
      this.parallaxLayers.push({ sprite: bg, speed: layer.speed });
    });
  }

  updateParallax() {
    if (!this.player || !this.player.body) return;
    const vx = this.player.body.velocity.x;
    this.parallaxLayers.forEach(({ sprite, speed }) => {
      sprite.tilePositionX += vx * speed * 0.016;
    });
  }

  // ─── Particles ─────────────────────────────────────────
  setupParticleEmitter(texture, config) {
    const emitter = this.add.particles(0, 0, texture, {
      x: { min: 0, max: this.cameras.main.width },
      y: config.y || { min: 0, max: this.cameras.main.height },
      lifespan: config.lifespan || 4000,
      speed: config.speed || { min: 10, max: 30 },
      scale: config.scale || { start: 0.5, end: 0 },
      alpha: config.alpha || { start: 0.6, end: 0 },
      frequency: config.frequency || 200,
      blendMode: config.blendMode || 'ADD',
      tint: config.tint || 0xffffff
    });
    emitter.setDepth(config.depth || 5);
    this.particleEmitters.push(emitter);
    return emitter;
  }

  // ─── Player ────────────────────────────────────────────
  createPlayer(x, y) {
    this.player = this.physics.add.sprite(x, y, 'player');
    this.player.setCollideWorldBounds(true);
    this.player.setDepth(10);
    this.player.setScale(2);

    // Player idle breathing tween
    this.tweens.add({
      targets: this.player,
      scaleY: 2.05,
      duration: 800,
      yoyo: true,
      repeat: -1,
      ease: 'Sine.easeInOut'
    });

    this.physics.add.collider(this.player, this.platforms);
    this.physics.add.overlap(this.player, this.npcGroup, this.onPlayerNearNPC, null, this);

    return this.player;
  }

  // ─── Movement ──────────────────────────────────────────
  handleMovement(speed = 200) {
    if (this.dialogueActive) {
      this.player.setVelocity(0);
      return;
    }

    const left = this.cursors.left.isDown || this.wasd.A.isDown;
    const right = this.cursors.right.isDown || this.wasd.D.isDown;
    const up = this.cursors.up.isDown || this.wasd.W.isDown;
    const down = this.cursors.down.isDown || this.wasd.S.isDown;

    if (left) {
      this.player.setVelocityX(-speed);
      this.player.setFlipX(true);
    } else if (right) {
      this.player.setVelocityX(speed);
      this.player.setFlipX(false);
    } else {
      this.player.setVelocityX(0);
    }

    if (up) {
      this.player.setVelocityY(-speed);
    } else if (down) {
      this.player.setVelocityY(speed);
    } else {
      this.player.setVelocityY(0);
    }

    // Walk bob tween
    if (left || right || up || down) {
      if (!this._walkTween || !this._walkTween.isPlaying()) {
        this._walkTween = this.tweens.add({
          targets: this.player,
          y: this.player.y - 3,
          duration: 150,
          yoyo: true,
          repeat: -1,
          ease: 'Sine.easeInOut'
        });
      }
    } else if (this._walkTween) {
      this._walkTween.stop();
      this._walkTween = null;
    }
  }

  // ─── NPC ───────────────────────────────────────────────
  createNPC(x, y, npcKey, name) {
    const npc = this.npcGroup.create(x, y, 'npc');
    npc.setScale(2);
    npc.setImmovable(true);
    npc.npcKey = npcKey;
    npc.npcName = name;
    npc.setDepth(10);

    // NPC idle animation
    this.tweens.add({
      targets: npc,
      y: npc.y - 4,
      duration: 1200,
      yoyo: true,
      repeat: -1,
      ease: 'Sine.easeInOut',
      delay: Math.random() * 500
    });

    // Name tag
    const nameTag = this.add.text(x, y - 55, name, {
      fontFamily: 'Arial',
      fontSize: '14px',
      color: '#ffffff',
      backgroundColor: '#00000088',
      padding: { x: 6, y: 2 }
    }).setOrigin(0.5).setDepth(11);

    return npc;
  }

  onPlayerNearNPC(player, npc) {
    // Show interaction prompt
    if (!npc._prompt) {
      npc._prompt = this.add.text(npc.x, npc.y - 70, '[E] Talk', {
        fontFamily: 'Arial',
        fontSize: '12px',
        color: '#ffdd00'
      }).setOrigin(0.5).setDepth(12);

      this.tweens.add({
        targets: npc._prompt,
        alpha: { from: 1, to: 0.4 },
        duration: 600,
        yoyo: true,
        repeat: -1
      });
    }
  }

  // ─── Dialogue System (Undertale-style) ─────────────────
  showDialogue(speaker, text, options = null) {
    if (this.dialogueActive) return;
    this.dialogueActive = true;

    const w = this.cameras.main.width;
    const h = this.cameras.main.height;

    // Dim overlay
    const overlay = this.add.rectangle(w / 2, h / 2, w, h, 0x000000, 0.4)
      .setDepth(20).setScrollFactor(0);

    // Dialogue box
    const box = this.add.image(w / 2, h - 100, 'dialogue-box')
      .setDepth(21).setScrollFactor(0).setScale(1.1);

    // Speaker name
    const nameText = this.add.text(w / 2 - 350, h - 170, speaker, {
      fontFamily: 'Arial',
      fontSize: '18px',
      color: '#ff69b4',
      fontStyle: 'bold'
    }).setDepth(22).setScrollFactor(0);

    // Typewriter text
    const bodyText = this.add.text(w / 2 - 340, h - 140, '', {
      fontFamily: 'Arial',
      fontSize: '16px',
      color: '#ffffff',
      wordWrap: { width: 680, useAdvancedWrap: true }
    }).setDepth(22).setScrollFactor(0);

    // Typewriter effect
    let charIndex = 0;
    const typeTimer = this.time.addEvent({
      delay: 30,
      callback: () => {
        if (charIndex < text.length) {
          bodyText.text += text[charIndex];
          charIndex++;
        } else {
          typeTimer.remove();
          if (options) {
            this.showChoices(options, w, h, overlay, box, nameText, bodyText);
          } else {
            this.addContinuePrompt(w, h, overlay, box, nameText, bodyText);
          }
        }
      },
      repeat: text.length - 1
    });

    // Skip typewriter with space
    this.input.keyboard.once('keydown-SPACE', () => {
      if (typeTimer.getRepeatCount() > 0) {
        typeTimer.remove();
        bodyText.text = text;
        if (options) {
          this.showChoices(options, w, h, overlay, box, nameText, bodyText);
        } else {
          this.addContinuePrompt(w, h, overlay, box, nameText, bodyText);
        }
      }
    });
  }

  addContinuePrompt(w, h, overlay, box, nameText, bodyText) {
    const prompt = this.add.text(w / 2, h - 35, '▼', {
      fontSize: '18px',
      color: '#ffffff'
    }).setOrigin(0.5).setDepth(22).setScrollFactor(0);

    this.tweens.add({
      targets: prompt,
      y: prompt.y + 6,
      duration: 500,
      yoyo: true,
      repeat: -1
    });

    this.input.keyboard.once('keydown-SPACE', () => {
      overlay.destroy();
      box.destroy();
      nameText.destroy();
      bodyText.destroy();
      prompt.destroy();
      this.dialogueActive = false;
    });
  }

  showChoices(options, w, h, overlay, box, nameText, bodyText) {
    const startY = h - 140;
    const choiceBtns = [];

    options.forEach((opt, i) => {
      const btn = this.add.image(w / 2, startY + i * 50, 'choice-btn')
        .setDepth(23).setScrollFactor(0).setInteractive({ useHandCursor: true });

      const label = this.add.text(w / 2, startY + i * 50, opt.text, {
        fontFamily: 'Arial',
        fontSize: '15px',
        color: '#ffffff'
      }).setOrigin(0.5).setDepth(24).setScrollFactor(0);

      btn.on('pointerover', () => {
        btn.setTexture('choice-btn-hover');
        label.setColor('#ff69b4');
      });
      btn.on('pointerout', () => {
        btn.setTexture('choice-btn');
        label.setColor('#ffffff');
      });
      btn.on('pointerdown', () => {
        // Clean up
        overlay.destroy();
        box.destroy();
        nameText.destroy();
        bodyText.destroy();
        choiceBtns.forEach(b => { b.btn.destroy(); b.label.destroy(); });
        this.dialogueActive = false;

        // Execute callback
        if (opt.callback) opt.callback();
      });

      choiceBtns.push({ btn, label });
    });
  }
}
