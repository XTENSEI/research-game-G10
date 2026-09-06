/**
 * proc.js — procedural art generation. No sprite sheets needed.
 * Every texture the game needs (skies, terrain, props, characters) is
 * drawn at load time so each biome reads as a real place, not particles.
 *
 * Palette language: bold flat fills + dark outline. Characters share one
 * body template; palette + accessory make each one distinct.
 */

const OUT = 0x241028;

// ─── tiny helpers ───────────────────────────────────────────

function freshGfx(scene, w, h) {
  const g = scene.make.graphics({ x: 0, y: 0, add: false });
  return { g, w, h };
}

function commit(scene, g, key, w, h) {
  g.generateTexture(key, w, h);
  g.destroy();
}

function skyCanvas(scene, key, cols) {
  const w = 512, h = 576;
  const cv = document.createElement('canvas');
  cv.width = w; cv.height = h;
  const c = cv.getContext('2d');
  const g = c.createLinearGradient(0, 0, 0, h);
  cols.forEach((col, i) => g.addColorStop(i / (cols.length - 1), col));
  c.fillStyle = g;
  c.fillRect(0, 0, w, h);
  scene.textures.addCanvas(key, cv);
}

function starfield(scene, key, count, near) {
  const w = 1024, h = 576;
  const cv = document.createElement('canvas');
  cv.width = w; cv.height = h;
  const c = cv.getContext('2d');
  for (let i = 0; i < count; i++) {
    const x = Math.random() * w, y = Math.random() * h;
    const r = near ? 1 + Math.random() * 1.6 : 0.6 + Math.random() * 1.2;
    const a = 0.25 + Math.random() * (near ? 0.7 : 0.45);
    c.fillStyle = `rgba(${near ? '255' : '255'},${near ? '255' : '240'},${near ? '255' : '235'},${a.toFixed(2)})`;
    c.beginPath();
    c.arc(x, y, r, 0, Math.PI * 2);
    c.fill();
    if (near && Math.random() < 0.12) {
      c.fillStyle = `rgba(255,220,255,${(a * 0.8).toFixed(2)})`;
      const s = 2 + Math.random() * 2;
      c.fillRect(x - s / 2, y - 0.5, s, 1);
      c.fillRect(x - 0.5, y - s / 2, 1, s);
    }
  }
  scene.textures.addCanvas(key, cv);
}

// ─── public entry ───────────────────────────────────────────

export function generateArt(scene) {
  createSkies(scene);
  createTerrain(scene);
  createCandyAssets(scene);
  createAquaAssets(scene);
  createCosmaAssets(scene);
  createHouseAssets(scene);
  createCharacters(scene);
  createMisc(scene);
}

function createSkies(scene) {
  skyCanvas(scene, 'sky-candy', ['#ff8ec8', '#ffd1ec', '#fff1db']);
  skyCanvas(scene, 'sky-aqua', ['#041321', '#07304f', '#0c4f73']);
  skyCanvas(scene, 'sky-cosma', ['#030017', '#0a0630', '#180c44']);
  skyCanvas(scene, 'sky-house', ['#2c1407', '#4a220e', '#5f2f13']);
  skyCanvas(scene, 'sky-crumble', ['#280a07', '#52150a', '#741b0a']);
  skyCanvas(scene, 'sky-home', ['#f6e3b6', '#f7ecd0', '#fbf3e1']);
  starfield(scene, 'stars-far', 220, false);
  starfield(scene, 'stars-near', 130, true);
}

// ─── terrain tiles (seamless base colour + contained speckle) ──

function createTerrain(scene) {
  // Candy lemon grass
  tile(scene, 'tile-candy', 0xfbef9a, (g, w, h) => {
    g.fillStyle(0xffffff, 0.25); g.fillRect(0, 0, w, 3);
    g.fillStyle(0xff8fc9, 0.5);
    for (let i = 0; i < 6; i++) g.fillCircle(3 + Math.random() * (w - 6), 14 + Math.random() * (h - 20), 1.3);
  });
  // Aquarius sand
  tile(scene, 'tile-aqua', 0x2a5b6e, (g, w, h) => {
    g.fillStyle(0x3d7288, 0.6); g.fillRect(0, 0, w, 2);
    g.fillStyle(0x7fc7d9, 0.3);
    for (let i = 0; i < 7; i++) g.fillCircle(2 + Math.random() * (w - 4), 6 + Math.random() * (h - 12), 1.1);
  });
  // Neon metal slab
  tile(scene, 'tile-neon', 0x151030, (g, w, h) => {
    g.fillStyle(0x5a4aff, 0.9); g.fillRect(0, 0, w, 2);
    g.fillStyle(0x8f7bff, 0.35); g.fillRect(0, 3, w, 2);
    g.fillStyle(0x2a1f55, 1); g.fillRect(0, 5, w, h - 5);
    g.fillStyle(0x0e0a20, 1); g.fillRect(0, h - 4, w, 4);
    g.fillStyle(0x6b5bff, 0.25);
    for (let i = 0; i < 4; i++) g.fillRect(4 + i * 8 + Math.random() * 2, h - 10, 2, 2);
  });
  // House planks
  tile(scene, 'tile-house', 0x6e3f1e, (g, w, h) => {
    g.fillStyle(0x000000, 0.18); g.fillRect(0, 0, w, 1);
    g.fillStyle(0x8a542a, 0.7); g.fillRect(0, h - 5, w, 4);
    g.fillStyle(0x4a2a12, 1); g.fillRect(w - 2, 0, 2, h);
    g.fillStyle(0x9a6a3a, 0.35); g.fillRect(3, 8, w - 8, 3);
  });
}

function tile(scene, key, base, paint) {
  const w = 64, h = 64;
  const { g } = freshGfx(scene, w, h);
  g.fillStyle(base, 1);
  g.fillRect(0, 0, w, h);
  paint(g, w, h);
  commit(scene, g, key, w, h);
}

function prop(scene, key, w, h, draw) {
  const { g } = freshGfx(scene, w, h);
  draw(g, w, h);
  commit(scene, g, key, w, h);
}

// ─── Candy Land ─────────────────────────────────────────────

function createCandyAssets(scene) {
  // Lollipop bush / prop
  prop(scene, 'lollipop', 64, 96, (g, w, h) => {
    g.fillStyle(0xf2e6d0, 1); g.fillRoundedRect(w / 2 - 3, h / 2, 6, h / 2 - 2, 3);
    const cx = w / 2, cy = 34, r = 26;
    g.fillStyle(0xff5f9e, 1); g.fillCircle(cx, cy, r);
    g.lineStyle(3, 0xffe0ee, 1);
    g.beginPath(); g.arc(cx, cy, 17, 0.2, 1.6); g.strokePath();
    g.beginPath(); g.arc(cx, cy, 17, 2.9, 4.4); g.strokePath();
    g.beginPath(); g.arc(cx, cy, 8, 0, Math.PI * 2); g.strokePath();
    g.fillStyle(0xffffff, 0.5); g.fillCircle(cx - 9, cy - 10, 4);
  });
  prop(scene, 'lollipop-blue', 64, 96, (g, w, h) => {
    g.fillStyle(0xf2e6d0, 1); g.fillRoundedRect(w / 2 - 3, h / 2, 6, h / 2 - 2, 3);
    const cx = w / 2, cy = 34, r = 26;
    g.fillStyle(0x3ec7ff, 1); g.fillCircle(cx, cy, r);
    g.lineStyle(3, 0xd8f4ff, 1);
    g.beginPath(); g.arc(cx, cy, 17, 0.2, 1.6); g.strokePath();
    g.beginPath(); g.arc(cx, cy, 17, 2.9, 4.4); g.strokePath();
    g.beginPath(); g.arc(cx, cy, 8, 0, Math.PI * 2); g.strokePath();
    g.fillStyle(0xffffff, 0.5); g.fillCircle(cx - 9, cy - 10, 4);
  });
  // Candy cane tree
  prop(scene, 'candy-tree', 72, 132, (g, w, h) => {
    const cx = w / 2;
    g.fillStyle(0xf6efe0, 1); g.fillRoundedRect(cx - 8, 30, 16, h - 32, 5);
    g.fillStyle(0xff3b5c, 1);
    for (let y = 36; y < h - 6; y += 14) g.fillRect(cx - 8, y, 16, 6);
    g.lineStyle(14, 0xff3b5c, 1);
    g.beginPath(); g.arc(cx, 30, 18, Math.PI, 0, false); g.strokePath();
    g.lineStyle(4, 0xffffff, 0.9);
    for (let a = Math.PI; a > 0; a -= 0.9) {
      const x1 = cx + Math.cos(a) * 16, y1 = 30 - Math.sin(a) * 16;
      g.beginPath(); g.arc(cx, 30, 16, a - 0.5, a); g.strokePath();
      g.lineBetween(cx + Math.cos(a) * 14, y1, cx + Math.cos(a) * 18, y1);
    }
  });
  // Cotton candy cloud puff (seamless edges)
  prop(scene, 'cloud-cotton', 256, 96, (g, w, h) => {
    for (let i = 0; i < 5; i++) {
      g.fillStyle(0xffb6e0, 0.9);
      const px = i * 52;
      g.fillCircle(px, 60 + (i % 2) * 6, 26);
      g.fillCircle(px + 24, 48, 20);
      g.fillCircle(px - 12, 50, 18);
      g.fillStyle(0xffd6ef, 0.9);
      g.fillCircle(px + 6, 46, 14);
    }
  });
  // Distant castle silhouette
  prop(scene, 'castle-far', 260, 220, (g, w, h) => {
    const c = 0x9c4a7d;
    g.fillStyle(c, 1);
    g.fillRect(20, 120, 220, 100);
    g.fillRect(34, 60, 44, 70);
    g.fillRect(182, 60, 44, 70);
    g.fillRect(78, 84, 100, 50);
    g.fillTriangle(56, 60, 78, 30, 100, 60);
    g.fillTriangle(156, 60, 178, 30, 200, 60);
    g.fillTriangle(78, 84, 128, 52, 178, 84);
    g.fillTriangle(20, 120, 56, 92, 92, 120);
    g.fillTriangle(168, 120, 204, 92, 240, 120);
    g.fillStyle(0xffd9f2, 0.9); g.fillRect(40, 84, 10, 16); g.fillRect(210, 84, 10, 16);
    g.fillStyle(0xff5f9e, 1); g.fillRect(52, 18, 8, 14); g.fillRect(196, 18, 8, 14);
  });
  // Lemon "hills" band for parallax (silhouette tint used in scene)
  prop(scene, 'ridge-candy', 512, 140, (g, w, h) => {
    g.fillStyle(0xf5cf5c, 0.9);
    let x = -20;
    while (x < w) {
      const r = 90 + Math.random() * 60;
      g.fillCircle(x, h, r);
      x += r;
    }
    g.fillRect(0, h - 30, w, 30);
  });
}

// ─── Aquarius ───────────────────────────────────────────────

function createAquaAssets(scene) {
  // Coral
  prop(scene, 'coral', 72, 80, (g, w, h) => {
    g.fillStyle(0xff6f9c, 1);
    g.fillRoundedRect(w / 2 - 5, h / 2, 10, h / 2, 5);
    for (let i = 0; i < 4; i++) {
      const bx = 16 + i * 14, by = 26 + (i % 2) * 10;
      g.fillCircle(bx, by, 8);
      g.fillStyle(0xff9dc0, 1); g.fillCircle(bx + 2, by - 2, 5);
      g.fillStyle(0xff6f9c, 1);
    }
  });
  // Seaweed
  prop(scene, 'seaweed', 56, 96, (g, w, h) => {
    const x = w / 2;
    g.fillStyle(0x2ea66a, 1);
    g.fillRoundedRect(x - 7, 20, 14, h - 20, 6);
    g.fillStyle(0x3fce8c, 1);
    g.fillCircle(x, 26, 8);
    g.fillStyle(0x228a52, 1);
    g.fillCircle(x, 46, 7);
    g.fillCircle(x, 66, 6);
  });
  // Kelp tall
  prop(scene, 'kelp', 70, 150, (g, w, h) => {
    g.fillStyle(0x1f9e63, 0.95);
    for (let y = 6; y < h; y += 22) {
      const x = w / 2 + (Math.sin(y / 16) * 8);
      g.fillEllipse(x, y, 16, 16);
    }
  });
  // Rock
  prop(scene, 'rock', 64, 52, (g, w, h) => {
    g.fillStyle(0x6a7f8c, 1);
    g.fillCircle(22, h - 8, 16);
    g.fillCircle(44, h - 10, 20);
    g.fillCircle(33, h - 24, 13);
    g.fillStyle(0x8ba4b0, 0.8); g.fillCircle(30, h - 22, 7);
  });
  // Shipwreck hull silhouette
  prop(scene, 'shipwreck', 220, 150, (g, w, h) => {
    g.fillStyle(0x3a2a1c, 1);
    g.fillRoundedRect(30, 70, 170, 60, 14);
    g.fillRoundedRect(30, 70, 60, 30, 8);
    g.fillRoundedRect(120, 70, 80, 30, 8);
    g.fillStyle(0x2a1e12, 1);
    g.fillTriangle(10, 130, 30, 60, 60, 130);
    g.fillStyle(0x533d28, 1);
    g.fillRoundedRect(80, 20, 8, 60, 4);
    g.fillRoundedRect(120, 20, 8, 60, 4);
    g.fillRect(66, 16, 76, 26);
    g.fillStyle(0x6d5138, 1);
    for (let i = 0; i < 3; i++) g.fillRect(90 + i * 24, 110, 14, 4);
    g.fillStyle(0x1a140e, 0.55); g.fillRect(30, 78, 170, 6);
  });
  // Coral ridge silhouette band
  prop(scene, 'ridge-aqua', 512, 130, (g, w, h) => {
    g.fillStyle(0x123f54, 0.9);
    let x = -10;
    while (x < w) {
      const r = 80 + Math.random() * 70;
      g.fillCircle(x, h, r);
      x += r * 0.8;
    }
    g.fillRect(0, h - 26, w, 26);
  });
}

// ─── Cosma ──────────────────────────────────────────────────

function createCosmaAssets(scene) {
  // Neon 4-point star
  prop(scene, 'neon-star', 64, 64, (g, w, h) => {
    const cx = w / 2, cy = h / 2;
    const c1 = 0x3a2aff, c2 = 0x9f8dff, c3 = 0xffffff;
    g.fillStyle(c1, 1);
    g.fillTriangle(cx, cy - 30, cx - 10, cy, cx + 10, cy);
    g.fillTriangle(cx, cy + 30, cx - 10, cy, cx + 10, cy);
    g.fillTriangle(cx - 30, cy, cx, cy - 10, cx, cy + 10);
    g.fillTriangle(cx + 30, cy, cx, cy - 10, cx, cy + 10);
    g.fillStyle(c2, 1);
    g.fillCircle(cx, cy, 12);
    g.fillStyle(c3, 1);
    g.fillCircle(cx, cy, 6);
  });
  prop(scene, 'neon-star-gold', 64, 64, (g, w, h) => {
    const cx = w / 2, cy = h / 2;
    g.fillStyle(0xffcc00, 1);
    g.fillTriangle(cx, cy - 30, cx - 10, cy, cx + 10, cy);
    g.fillTriangle(cx, cy + 30, cx - 10, cy, cx + 10, cy);
    g.fillTriangle(cx - 30, cy, cx, cy - 10, cx, cy + 10);
    g.fillTriangle(cx + 30, cy, cx, cy - 10, cx, cy + 10);
    g.fillStyle(0xfff2b0, 1);
    g.fillCircle(cx, cy, 12);
    g.fillStyle(0xffffff, 1);
    g.fillCircle(cx, cy, 6);
  });
  // Ringed planet
  prop(scene, 'neon-ring', 96, 72, (g, w, h) => {
    g.fillStyle(0x9a4aff, 0.9);
    g.fillCircle(w / 2, h / 2, 22);
    g.fillStyle(0xcfa8ff, 1);
    g.fillCircle(w / 2 - 6, h / 2 - 6, 7);
    g.lineStyle(5, 0x66e0ff, 0.95);
    g.strokeEllipse(w / 2, h / 2 + 4, 84, 26);
  });
  // Floating stone with glow belly
  prop(scene, 'floatstone', 90, 60, (g, w, h) => {
    g.fillStyle(0x2c2452, 1);
    g.fillCircle(24, 26, 15);
    g.fillCircle(48, 32, 18);
    g.fillCircle(70, 24, 13);
    g.fillCircle(46, 18, 14);
    g.fillStyle(0x6b5bff, 1);
    g.fillEllipse(46, 46, 44, 8);
    g.fillStyle(0x9f8dff, 0.9);
    g.fillEllipse(40, 44, 26, 5);
  });
  // Neon crystal
  prop(scene, 'crystal-neon', 56, 84, (g, w, h) => {
    const cx = w / 2;
    g.fillStyle(0x37e0ff, 1);
    g.fillTriangle(cx, h - 8, cx - 20, h - 26, cx - 8, h - 8);
    g.fillStyle(0x2a9fff, 1);
    g.fillTriangle(cx, h - 8, cx + 20, h - 26, cx + 8, h - 8);
    g.fillStyle(0x8ef2ff, 1);
    g.fillTriangle(cx, h - 8, cx, h - 44, cx - 10, h - 22);
    g.fillStyle(0xffffff, 0.85);
    g.fillTriangle(cx, h - 8, cx, h - 30, cx - 5, h - 18);
  });
  // Distant ruined towers band
  prop(scene, 'ridge-cosma', 512, 150, (g, w, h) => {
    g.fillStyle(0x0d0a24, 0.95);
    let x = 0;
    while (x < w) {
      const tw = 60 + Math.random() * 70;
      const th = 40 + Math.random() * 70;
      g.fillRect(x, h - th, tw, th);
      if (Math.random() < 0.5) g.fillRect(x + tw / 2 - 4, h - th - 22, 8, 22);
      x += tw + 8 + Math.random() * 30;
    }
    g.fillRect(0, h - 12, w, 12);
  });
}

// ─── House / ending ─────────────────────────────────────────

function createHouseAssets(scene) {
  prop(scene, 'window', 110, 110, (g, w, h) => {
    g.fillStyle(0x2c1407, 1);
    g.fillRoundedRect(5, 5, w - 10, h - 10, 8);
    g.fillStyle(0xffd27a, 1);
    g.fillRect(12, 14, w - 24, h - 26);
    g.lineStyle(4, 0x8a4a1c, 1);
    g.lineBetween(12, h / 2, w - 12, h / 2);
    g.lineBetween(w / 2, 14, w / 2, h - 12);
    g.fillStyle(0xfff2cc, 0.25);
    g.fillRect(14, 16, w / 2 - 24, h - 30);
  });
  prop(scene, 'bed', 150, 90, (g, w, h) => {
    g.fillStyle(0x5a3418, 1); g.fillRect(6, h - 22, w - 12, 16);
    g.fillStyle(0xff7ab0, 1); g.fillRoundedRect(6, 12, w - 12, 46, 8);
    g.fillStyle(0xffffff, 0.9); g.fillRoundedRect(10, 16, w - 20, 18, 6);
    g.fillStyle(0x8a4a2a, 1); g.fillCircle(w - 22, h - 26, 14);
    g.fillStyle(0x6a3518, 1); g.fillRect(w - 24, h - 30, 4, 6);
  });
  prop(scene, 'shelf', 200, 120, (g, w, h) => {
    g.fillStyle(0x6e3f1e, 1); g.fillRect(0, h - 40, w, 10);
    g.fillStyle(0x3a6fb5, 1); g.fillRect(8, h - 64, 8, 26);
    g.fillStyle(0x3a6fb5, 1); g.fillRect(60, h - 74, 8, 36);
    g.fillStyle(0xffcc44, 0.9); g.fillCircle(34, h - 70, 7);
    g.fillStyle(0x9a6a3a, 1); g.fillRect(90, h - 66, 34, 28);
    g.fillStyle(0xffeef2, 1); g.fillRect(92, h - 64, 30, 24);
  });
}

// ─── misc shared ────────────────────────────────────────────

function createMisc(scene) {
  prop(scene, 'talk-bubble', 96, 40, (g, w, h) => {
    g.fillStyle(0xffffff, 0.96);
    g.fillRoundedRect(0, 0, w, h - 8, 12);
    g.fillTriangle(w / 2 - 8, h - 12, w / 2 + 8, h - 12, w / 2, h);
    g.lineStyle(2, 0x241028, 1);
    g.strokeRoundedRect(0, 0, w, h - 8, 12);
  });
  prop(scene, 'hearts', 90, 30, (g, w, h) => {
    g.fillStyle(0xff4d6d, 1);
    const draw = (x, y, s) => {
      g.fillCircle(x - s * 0.5, y, s);
      g.fillCircle(x + s * 0.5, y, s);
      g.fillTriangle(x - s, y, x + s, y, x, y + s * 1.6);
    };
    draw(18, 12, 8);
    g.fillStyle(0x555, 0.5); g.fillCircle(54, 14, 5);
    g.fillCircle(72, 14, 5);
  });
}

// ─── characters ─────────────────────────────────────────────

/**
 * Body template: front-facing traveller 40x56.
 * pal: { skin, skin2, hair, hair2, shirt, shirt2, pants, shoes, acc, acc2 }
 * opts: { hair: 'short'|'long'|'bald'|'gray'|'crown'|'flower'|'hood',
 *         beard: bool, cookie: bool (gingerbread), robe: bool }
 */
export function createCharacters(scene) {
  const chars = {
    player: {
      pal: { skin: 0xf0a56a, hair: 0x2b1a30, shirt: 0x3d7bd9, shirt2: 0x6ea2e8, pants: 0x2c3d66, shoes: 0x4a2a1e, acc: 0xffd24d },
      o: {}
    },
    'char-jin': {
      pal: { skin: 0xd9893c, hair: 0xd9893c, shirt: 0xd9893c, shirt2: 0xf0a75e, pants: 0xd9893c, shoes: 0xd9893c },
      o: { cookie: true }
    },
    'char-gillbert': {
      pal: { skin: 0xd8a878, hair: 0xc7c2b8, shirt: 0x7a5a3a, shirt2: 0x9a7a52, pants: 0x4a3822, shoes: 0x3a2a18, acc: 0xd8d4cc },
      o: { beard: true, hair: 'gray' }
    },
    'char-scalleon': {
      pal: { skin: 0xbff0d0, hair: 0x1f9e63, shirt: 0x2ea66a, shirt2: 0x3fce8c, pants: 0x1f6a44, shoes: 0x143a26 },
      o: { hair: 'fin' }
    },
    'char-market-lady': {
      pal: { skin: 0xffb98a, hair: 0x8a4a2a, shirt: 0xff8a6a, shirt2: 0xffb48a, pants: 0x8a4a3a, shoes: 0x5a2a1a, acc: 0xffdd4d },
      o: { hair: 'flower' }
    },
    'char-nadia': {
      pal: { skin: 0xffc79a, hair: 0x6b3a1c, shirt: 0xff7ab0, shirt2: 0xffb3d6, pants: 0xff9dc4, shoes: 0x8a3a5a, acc: 0xffffff },
      o: { hair: 'long' }
    },
    'char-grandparent': {
      pal: { skin: 0xe0b48a, hair: 0xcfcfd6, shirt: 0x4a7a3a, shirt2: 0x6a9a52, pants: 0x2c3a22, shoes: 0x1c2414 },
      o: { hair: 'gray', beard: true, robe: true }
    },
    'char-kween': {
      pal: { skin: 0xffd9de, hair: 0x3a1a3a, shirt: 0xb02a6a, shirt2: 0xea4d9a, pants: 0x6a1a44, shoes: 0x4a0e30, acc: 0xffd24d },
      o: { hair: 'crown', robe: true }
    },
    'char-matt': {
      pal: { skin: 0xf0c49a, hair: 0x241a26, shirt: 0x4a4a66, shirt2: 0x66668a, pants: 0x33334a, shoes: 0x1a1a26, acc: 0xdddddd },
      o: {}
    },
    'char-guard': {
      pal: { skin: 0xf0b98a, hair: 0x3a1a1a, shirt: 0xe04848, shirt2: 0xfff2f2, pants: 0x8a2a2a, shoes: 0x4a1414, acc: 0xffd24d },
      o: { hair: 'guard' }
    }
  };

  for (const [key, def] of Object.entries(chars)) {
    drawChar(scene, key, def.pal, def.o);
    drawChar(scene, key + '-w1', def.pal, def.o, 1);
    drawChar(scene, key + '-w2', def.pal, def.o, 2);
  }
  // Generic NPC fallback still exists
  drawChar(scene, 'npc', { skin: 0xd9893c, hair: 0x4a2a1a, shirt: 0x5a8a4a, shirt2: 0x7aab66, pants: 0x3a5a30, shoes: 0x2a3a1e }, {});
}

function drawChar(scene, key, pal, o, walk = 0) {
  const w = 40, h = 58;
  const { g } = freshGfx(scene, w, h);
  const cx = w / 2;

  // legs + feet (walk: 0 idle, 1 step-wide, 2 step-together)
  const legStep = walk === 1 ? 3 : 0;
  const pant = pal.pants, shoe = pal.shoes, skin = pal.skin;

  g.lineStyle(2, OUT, 1);
  // feet
  g.fillStyle(shoe, 1);
  if (walk === 0) { g.fillRoundedRect(12, 50, 7, 6, 2); g.fillRoundedRect(22, 50, 7, 6, 2); }
  else if (walk === 1) { g.fillRoundedRect(9, 51, 7, 5, 2); g.fillRoundedRect(25, 51, 7, 5, 2); }
  else { g.fillRoundedRect(13, 50, 7, 5, 2); g.fillRoundedRect(21, 50, 7, 5, 2); }
  // pants legs
  g.fillStyle(pant, 1);
  g.fillRoundedRect(12, 40, 8, 12, 2);
  g.fillRoundedRect(20, 40, 8, 12, 2);

  if (o.cookie) {
    // gingerbread cookie body
    g.fillStyle(pal.shirt, 1);
    g.fillEllipse(cx, 34, 22, 24);
    g.lineStyle(2, 0x5a2f08, 1);
    g.strokeEllipse(cx, 34, 22, 24);
    g.fillStyle(0xffffff, 1);
    g.fillCircle(cx - 6, 28, 2.4);
    g.fillCircle(cx + 6, 28, 2.4);
    g.fillCircle(cx, 36, 2.4);
    g.fillCircle(cx, 42, 2.4);
    g.lineStyle(2, 0xffffff, 1);
    g.beginPath(); g.arc(cx, 30, 5, 0.3, Math.PI - 0.3, false); g.strokePath();
  } else {
    // arms (skin)
    g.fillStyle(skin, 1);
    g.fillRoundedRect(6, 27, 6, 13, 3);
    g.fillRoundedRect(28, 27, 6, 13, 3);
    // torso
    g.fillStyle(pal.shirt, 1);
    g.fillRoundedRect(9, 25, 22, 17, 5);
    g.lineStyle(2, OUT, 1);
    g.strokeRoundedRect(9, 25, 22, 17, 5);
    if (pal.shirt2) {
      g.lineStyle(2, pal.shirt2, 1);
      g.lineBetween(10, 30, 30, 30);
    }
    // robe over torso (grandparent/kween)
    if (o.robe) {
      g.fillStyle(pal.shirt2, 1);
      g.fillTriangle(12, 25, 28, 25, cx, 47);
      g.lineStyle(2, pal.shirt2, 1);
      g.lineBetween(14, 40, 26, 40);
    }
  }

  // head
  const hy = o.cookie ? 17 : 17;
  if (!o.cookie) {
    g.fillStyle(skin, 1);
    g.fillCircle(cx, hy, 11);
    g.lineStyle(2, OUT, 1);
    g.strokeCircle(cx, hy, 11);
    // neck
    g.fillStyle(skin, 1);
    g.fillRect(17, 24, 6, 3);
  }

  // hair / accessory
  const hairC = pal.hair;
  g.fillStyle(hairC, 1);
  const style = o.hair || 'short';
  if (o.cookie) {
    // icing hair cap
    g.fillStyle(0xffffff, 1);
    g.fillEllipse(cx, 8, 20, 7);
    g.lineStyle(2, 0xffffff, 1);
    g.lineBetween(cx - 10, 8, cx + 10, 8);
  } else if (style === 'short' || style === 'fin') {
    g.fillEllipse(cx, 8, 20, 8);
    g.fillRect(cx - 10, 6, 20, 6);
    if (style === 'fin') {
      g.fillStyle(pal.hair, 1);
      g.fillTriangle(cx - 10, 10, cx - 18, 0, cx - 4, 12);
      g.fillTriangle(cx + 10, 10, cx + 18, 0, cx + 4, 12);
    }
  } else if (style === 'long') {
    g.fillEllipse(cx, 8, 18, 10);
    g.fillRect(5, 6, 10, 26);
    g.fillRect(25, 6, 10, 26);
  } else if (style === 'gray') {
    g.fillEllipse(cx, 8, 20, 9);
    g.fillRect(6, 8, 8, 8);
    g.fillRect(26, 8, 8, 8);
  } else if (style === 'crown') {
    g.fillStyle(pal.acc, 1);
    g.fillTriangle(cx - 10, 2, cx + 10, 2, cx, -6);
    g.fillTriangle(cx - 14, 6, cx - 4, -2, cx - 8, 6);
    g.fillStyle(0xff3b5c, 1);
    g.fillCircle(cx, 0, 2);
    g.fillStyle(pal.acc, 1);
  } else if (style === 'flower') {
    g.fillEllipse(cx, 8, 18, 8);
    g.fillStyle(pal.acc, 1);
    for (let i = 0; i < 5; i++) {
      const a = (i / 5) * Math.PI * 2;
      g.fillCircle(cx + 5 + Math.cos(a) * 5, 0 + Math.sin(a) * 5, 3);
    }
    g.fillStyle(0xff8f9e, 1);
    g.fillCircle(cx + 5, 0, 3);
  } else if (style === 'guard') {
    g.fillEllipse(cx, 8, 20, 8);
    g.fillStyle(pal.acc, 1);
    g.fillRoundedRect(cx - 11, -4, 22, 9, 3);
    g.fillStyle(0xe04848, 1);
    g.fillTriangle(cx - 11, -4, cx + 11, -4, cx, -12);
  } else { // hood or default spiky
    g.fillEllipse(cx, 8, 20, 9);
    g.fillTriangle(cx - 9, 10, cx - 13, -2, cx - 2, 8);
    g.fillTriangle(cx + 9, 10, cx + 13, -2, cx + 2, 8);
  }

  if (o.beard) {
    g.fillStyle(pal.acc || pal.hair, 1);
    g.fillRoundedRect(cx - 9, 18, 18, 12, 6);
  }

  if (!o.cookie) {
    // face details
    g.fillStyle(0xffffff, 1);
    g.fillCircle(15.5, 17, 2.6);
    g.fillCircle(24.5, 17, 2.6);
    g.fillStyle(pal.hair2 || 0x1a0a20, 1);
    g.fillCircle(15.5, 17, 1.3);
    g.fillCircle(24.5, 17, 1.3);
    // brows
    g.lineStyle(1.6, pal.hair2 || 0x1a0a20, 1);
    g.lineBetween(13, 12.6, 18, 12.8);
    g.lineBetween(27, 12.6, 22, 12.8);
    // mouth
    g.lineStyle(1.6, 0x7a3a2a, 1);
    g.beginPath(); g.arc(cx, 21, 3.4, 0.3, Math.PI - 0.3, false); g.strokePath();
  }

  commit(scene, g, key, w, h);
}
