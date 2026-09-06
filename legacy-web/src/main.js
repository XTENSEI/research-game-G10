import Phaser from 'phaser';
import { BootScene } from './scenes/BootScene.js';
import { PreloaderScene } from './scenes/PreloaderScene.js';
import { IntroScene } from './scenes/IntroScene.js';
import { CandyLandScene } from './scenes/CandyLandScene.js';
import { AquariusScene } from './scenes/AquariusScene.js';
import { CosmaScene } from './scenes/CosmaScene.js';
import { EndingScene } from './scenes/EndingScene.js';

const config = {
  type: Phaser.AUTO,
  parent: 'game-container',
  width: 1024,
  height: 576,
  pixelArt: false,
  roundPixels: true,
  physics: {
    default: 'arcade',
    arcade: { gravity: { y: 0 }, debug: false }
  },
  scene: [
    BootScene,
    PreloaderScene,
    IntroScene,
    CandyLandScene,
    AquariusScene,
    CosmaScene,
    EndingScene
  ]
};

const game = new Phaser.Game(config);

// Auto-focus canvas for keyboard input
game.canvas.setAttribute('tabindex', '0');
game.canvas.focus();

// Expose for testing
globalThis.__game = game;

export default game;
