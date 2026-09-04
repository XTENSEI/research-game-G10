import Phaser from 'phaser';
import { DialogueBox } from '../ui/DialogueBox.js';
import { audioManager } from '../audio/AudioManager.js';

/** Shared geometry: characters stand with feet on this ground line. */
export const GROUND_Y = 470;
export const CHAR_SCALE = 1.9;
const CHAR_HALF = (58 * CHAR_SCALE) / 2; // display half-height
const SHADOW_DY = CHAR_HALF + 4;

/**
 * BaseScene with WASD/arrows, parallax, particles, Undertale-style dialogue.
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
    this.uiAccent = '#ff5f9e'; // theme color consumed by DialogueBox/panels
    this.uiAccentDark = '#7a2a4a';
  }

  init(data) {
    this.gameState = data.gameState || {
      hasHorse: false, completedRiddles: [], completedPuzzles: [],
      inventory: [], health: 100, lives: 3, battleCorrect: 0, battleWrong: 0
    };
  }

  setTheme(accent, accentDark) {
    this.uiAccent = accent;
    if (accentDark) this.uiAccentDark = accentDark;
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

  // ─── Scenery helpers ──

  /** Fullscreen sky gradient. */
  setupSky(skyKey) {
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;
    this.sky = this.add.image(w / 2, h / 2, skyKey).setScrollFactor(0).setDepth(-20);
    return this.sky;
  }

  /** Parallax list: {key, y, speed, depth, tint, alpha, height} as repeating bands. */
  setupParallax(layers) {
    this.parallaxLayers = [];
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;

    layers.forEach((layer) => {
      const bandH = layer.height ?? h;
      const bg = this.add.tileSprite(w / 2, layer.y ?? h / 2, w, bandH, layer.key)
        .setOrigin(0.5).setScrollFactor(0).setDepth(layer.depth ?? 0);
      if (layer.tint) bg.setTint(layer.tint);
      if (layer.alpha) bg.setAlpha(layer.alpha);
      this.parallaxLayers.push({ sprite: bg, speed: layer.speed });
    });
  }

  /** Add a single parallax band (helper). */
  addBand(key, y, height, depth, alpha, speed) {
    const w = this.cameras.main.width;
    const bg = this.add.tileSprite(w / 2, y, w, height, key)
      .setOrigin(0.5).setScrollFactor(0).setDepth(depth).setAlpha(alpha ?? 1);
    this.parallaxLayers.push({ sprite: bg, speed: speed ?? 0.1 });
    return bg;
  }

  updateParallax() {
    if (!this.player || !this.player.body) return;
    const vx = this.player.body.velocity.x;
    this.parallaxLayers.forEach(({ sprite, speed }) => {
      sprite.tilePositionX += vx * speed * 0.016;
    });
  }

  /** Ground band of tiled terrain + darker lip at the bottom. */
  setupGround(tileKey, top = GROUND_Y, lip = 0x1a0a10) {
    const w = this.cameras.main.width;
    const h = this.cameras.main.height;
    const gh = h - top;
    this.ground = this.add.tileSprite(w / 2, top + gh / 2, w, gh, tileKey)
      .setScrollFactor(0).setDepth(6);
    this.add.rectangle(w / 2, h - 7, w, 14, lip, 1).setScrollFactor(0).setDepth(7);
    return this.ground;
  }

  /** Prop standing on the ground line. */
  placeProp(key, x, scale = 1, depth = 8, flip = false) {
    const img = this.add.image(x, GROUND_Y - 2, key).setDepth(depth).setScale(scale);
    if (flip) img.setFlipX(true);
    return img;
  }

  setupVignette(alpha = 0.5, depth = 89) {
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

  createPlayer(x, y, texKey = 'player') {
    const sprite = this.physics.add.sprite(x, y, texKey);
    sprite.setCollideWorldBounds(true);
    sprite.setDepth(10);
    sprite.setScale(CHAR_SCALE);
    this.player = sprite;

    // Gentle idle breathing
    this.tweens.add({
      targets: sprite, scaleY: CHAR_SCALE + 0.04,
      duration: 1100, yoyo: true, repeat: -1, ease: 'Sine.easeInOut'
    });

    this.player._baseY = y;
    this.player._idleTex = texKey;
    this.player._walkTex = [texKey + '-w1', texKey + '-w2'];
    this.player._moving = false;

    // Shadow
    this.player.shadow = this.add.ellipse(x, y + SHADOW_DY, 30, 9, 0x000000, 0.28).setDepth(9);
    return this.player;
  }

  // ─── Movement ──

  handleMovement(speed = 190) {
    if (this.dialogueActive || !this.player) {
      if (this.player) {
        this.player.setVelocity(0);
        if (this.player._moving) this._endWalk();
      }
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

    if (up) this.player.setVelocityY(-speed * 0.85);
    else if (down) this.player.setVelocityY(speed * 0.85);
    else this.player.setVelocityY(0);

    const now = this.time.now;
    if (moving) {
      if (!this.player._moving) this.player._moving = true;
      // 2-frame walk cycle + bob
      const texs = this.player._walkTex;
      const frame = Math.floor(now / 140) % 2;
      const target = texs[frame];
      if (this.textures.exists(target) && this.player.texture.key !== target) {
        this.player.setTexture(target);
      }
      this.player.y = this.player._baseY + (frame === 1 ? 2 : -1);
      if (now - this._lastFootstep > 230) {
        audioManager.playFootstep();
        this._lastFootstep = now;
      }
    } else if (this.player._moving) {
      this._endWalk();
    }

    if (this.player.shadow) {
      this.player.shadow.x = this.player.x;
      this.player.shadow.y = this.player.y + SHADOW_DY;
    }
  }

  _endWalk() {
    this.player._moving = false;
    this.player.y = this.player._baseY;
    const idle = this.player._idleTex;
    if (this.textures.exists(idle)) this.player.setTexture(idle);
  }

  // ─── NPC ──

  createNPC(x, y, npcKey, name, portraitKey) {
    const tex = this.textures.exists(`char-${npcKey}`) ? `char-${npcKey}` : 'npc';
    const npc = this.npcGroup.create(x, y, tex);
    npc.setScale(CHAR_SCALE);
    npc.setImmovable(true);
    npc.npcKey = npcKey;
    npc.npcName = name;
    npc.portraitKey = portraitKey || `portrait-${npcKey}`;
    npc.setDepth(10);
    npc.talkable = true;
    npc._baseY = y;

    this.tweens.add({
      targets: npc, y: npc._baseY - 4,
      duration: 1400, yoyo: true, repeat: -1, ease: 'Sine.easeInOut',
      delay: Math.random() * 600
    });

    npc.shadow = this.add.ellipse(x, y + SHADOW_DY, 30, 9, 0x000000, 0.28).setDepth(9);

    // Name tag
    const nameTag = this.add.text(x, y - 70, name, {
      fontFamily: '"Courier New", monospace', fontSize: '13px',
      color: this.uiAccent, stroke: '#000000', strokeThickness: 3,
      backgroundColor: '#00000077', padding: { x: 8, y: 3 }
    }).setOrigin(0.5).setDepth(11);
    npc.nameTag = nameTag;

    // Talk prompt (shown when player is near)
    if (this.textures.exists('talk-bubble')) {
      const bubble = this.add.image(x, y - 96, 'talk-bubble').setDepth(12).setAlpha(0);
      const bubbleText = this.add.text(x, y - 96, '\u25B6', {
        fontFamily: '"Courier New", monospace', fontSize: '15px', fontStyle: 'bold',
        color: '#241028'
      }).setOrigin(0.5).setDepth(13).setAlpha(0);
      npc.bubble = bubble;
      npc.bubbleText = bubbleText;
    }

    return npc;
  }

  /** Show/hide talk bubbles over nearby NPCs (call from scene update). */
  updateNPCs() {
    if (!this.player) return;
    this.npcGroup.getChildren().forEach((npc) => {
      const near = Phaser.Math.Distance.Between(this.player.x, this.player.y, npc.x, npc.y) < 120;
      const show = near && npc.talkable && !this.dialogueActive;
      if (npc.bubble) {
        if (show && npc.bubble.alpha < 1) {
          this.tweens.add({ targets: [npc.bubble, npc.bubbleText], alpha: 1, duration: 120 });
        } else if (!show && npc.bubble.alpha > 0) {
          npc.bubble.setAlpha(0);
          npc.bubbleText.setAlpha(0);
        }
      }
    });
  }

  // ─── Dialogue wrapper ──
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
