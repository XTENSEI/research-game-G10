import Phaser from 'phaser';
import { DialogueBox } from '../ui/DialogueBox.js';
import { audioManager } from '../audio/AudioManager.js';

/**
 * Base scene with WASD/arrows, parallax, particles, Undertale-style dialogue.
 */
export class BaseScene extends Phaser.Scene {
  constructor(opts) {
    super(opts.key);
    this.biomeId = opts.biomeId || opts.key;
    this.player = null;
    this.cursors = null;
    this.wasd = null;
    this.npcGroup = null;
    this.dialogueActive = false;
    this.parallaxLayers = [];
    this.dialogueBox = null;
    this._lastFootstep = 0;
  }

  init(data) {
    this.gameState = data.gameState || {
      hasHorse: false, completedRiddles: [], completedPuzzles: [],
      inventory: [], health: 100, lives: 3, battleCorrect: 0, battleWrong: 0
    };
  }

  create() {
    audioManager.init();
    audioManager.resume();

    this.cursors = this.input.keyboard.createCursorKeys();
    this.wasd = {
      W: this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.W),
      A: this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.A),
      S: this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.S),
      D: this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.D)
    };

    this.npcGroup = this.physics.add.group();
    this.dialogueBox = new DialogueBox(this);
  }

  // ─── Parallax ──

  setupParallax(layers) {
    this.parallaxLayers = [];
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;

    layers.forEach((layer) => {
      const bg = this.add.tileSprite(0, layer.y ?? h / 2, w * 2, h, layer.key)
        .setOrigin(0.5).setScrollFactor(0).setDepth(layer.depth ?? 0);
      if (layer.tint) bg.setTint(layer.tint);
      if (layer.alpha) bg.setAlpha(layer.alpha);
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

  // ─── Atmosphere helpers ──

  setupGround(color, top = 470) {
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;
    this.ground = this.add.rectangle(w / 2, top + (h - top) / 2, w, h - top, color, 1).setDepth(6);
    return this.ground;
  }

  setupVignette(alpha = 0.55, depth = 89) {
    if (!this.textures.exists('vignette')) return null;
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;
    this.vignette = this.add.image(w / 2, h / 2, 'vignette')
      .setDisplaySize(w, h).setScrollFactor(0).setDepth(depth).setAlpha(alpha);
    return this.vignette;
  }

  // ─── Particles ──

  setupEmitter(texture, config) {
    const emitter = this.add.particles(0, 0, texture, {
      x: config.x ?? { min: 0, max: this.cameras.main.width },
      y: config.y ?? { min: 0, max: this.cameras.main.height },
      lifespan: config.lifespan ?? 4000,
      speed: config.speed ?? { min: 10, max: 30 },
      scale: config.scale ?? { start: 0.5, end: 0 },
      alpha: config.alpha ?? { start: 0.6, end: 0 },
      frequency: config.frequency ?? 200,
      blendMode: config.blendMode ?? 'ADD',
      tint: config.tint ?? 0xffffff
    });
    emitter.setDepth(config.depth ?? 5);
    return emitter;
  }

  // ─── Player ──

  createPlayer(x, y) {
    this.player = this.physics.add.sprite(x, y, 'player');
    this.player.setCollideWorldBounds(true);
    this.player.setDepth(10);
    this.player.setScale(2.5);

    // Idle breathing
    this.tweens.add({
      targets: this.player, scaleY: 2.55,
      duration: 1200, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
    });

    // Shadow
    this.player.shadow = this.add.ellipse(x, y + 24, 40, 10, 0x000000, 0.3).setDepth(9);

    if (this.platforms) this.physics.add.collider(this.player, this.platforms);
    return this.player;
  }

  // ─── Movement ──

  handleMovement(speed = 200) {
    if (this.dialogueActive || !this.player) {
      if (this.player) this.player.setVelocity(0);
      return;
    }

    const left = this.cursors.left.isDown || this.wasd.A.isDown;
    const right = this.cursors.right.isDown || this.wasd.D.isDown;
    const up = this.cursors.up.isDown || this.wasd.W.isDown;
    const down = this.cursors.down.isDown || this.wasd.S.isDown;
    const moving = left || right || up || down;

    if (left) { this.player.setVelocityX(-speed); this.player.setFlipX(true); }
    else if (right) { this.player.setVelocityX(speed); this.player.setFlipX(false); }
    else this.player.setVelocityX(0);

    if (up) this.player.setVelocityY(-speed);
    else if (down) this.player.setVelocityY(speed);
    else this.player.setVelocityY(0);

    if (moving) {
      if (!this._walkTween || !this._walkTween.isPlaying()) {
        this._walkTween = this.tweens.add({
          targets: this.player,
          scaleX: { from: 2.4, to: 2.6 }, scaleY: { from: 2.6, to: 2.4 },
          duration: 120, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
        });
      }
      const now = this.time.now;
      if (now - this._lastFootstep > 250) {
        audioManager.playFootstep();
        this._lastFootstep = now;
      }
    } else if (this._walkTween) {
      this._walkTween.stop();
      this._walkTween = null;
      this.player.setScale(2.5);
    }

    if (this.player.shadow) {
      this.player.shadow.x = this.player.x;
      this.player.shadow.y = this.player.y + 24;
    }
  }

  // ─── NPC ──

  createNPC(x, y, npcKey, name, portraitKey, tint) {
    const npc = this.npcGroup.create(x, y, 'npc');
    npc.setScale(2.5);
    npc.setImmovable(true);
    npc.npcKey = npcKey;
    npc.npcName = name;
    npc.portraitKey = portraitKey || `portrait-${npcKey}`;
    npc.setDepth(10);
    if (tint) npc.setTint(tint);

    this.tweens.add({
      targets: npc, y: npc.y - 5,
      duration: 1400, yoyo: true, repeat: -1, ease: 'Sine.easeInOut',
      delay: Math.random() * 600
    });

    npc.shadow = this.add.ellipse(x, y + 24, 40, 10, 0x000000, 0.3).setDepth(9);

    const nameTag = this.add.text(x, y - 60, name, {
      fontFamily: 'Arial, Helvetica, sans-serif', fontSize: '13px',
      color: '#ff69b4', stroke: '#000000', strokeThickness: 3,
      backgroundColor: '#00000066', padding: { x: 8, y: 3 }
    }).setOrigin(0.5).setDepth(11);
    npc.nameTag = nameTag;

    return npc;
  }

  // ─── Dialogue wrapper ──
  // showDialogue clears dialogueActive synchronously via onDestroy callback.
  // The next dialogue call in the callback chain will work immediately.

  showDialogue(speaker, text, options = null, portraitKey = null, expression = null) {
    if (this.dialogueActive) return;
    this.dialogueActive = true;
    this.dialogueBox.show(speaker, text, portraitKey, expression, options, () => {
      this.dialogueActive = false;
    });
  }

  transitionTo(sceneKey, fadeDuration = 1000, fadeColor = 0x000000) {
    audioManager.stopAllAmbient();
    this.cameras.main.fadeOut(fadeDuration, (fadeColor >> 16) & 0xff, (fadeColor >> 8) & 0xff, fadeColor & 0xff);
    this.cameras.main.once('camerafadeoutcomplete', () => {
      this.scene.start(sceneKey, { gameState: this.gameState });
    });
  }
}
