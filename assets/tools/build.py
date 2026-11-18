#!/usr/bin/env python3
"""Build game art from vendored Kenney CC0 assets.

Emits (committed, no python needed at runtime):
  assets/game/sheet_candy.png    full palette-remapped roguelike sheet (16px, 1px gutters)
  assets/game/tiles/<name>.png   selected 16x16 tiles
  assets/game/char/<name>.png    animated cast, uniform 96x144 canvas (2:3):
                                 <name>.png = stand, <name>_w1.png / _w2.png = walk cycle

Tiles layout facts (verified from the sheet): 16px cells on a 1px gutter grid,
57 columns, stride 17, origin (1,1). Tile id = row * 57 + col.
"""
import os
from PIL import Image

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
RL = os.path.join(ROOT, 'assets/vendor/kenney/roguelike/Spritesheet/roguelikeSheet_transparent.png')
OUT_SHEET = os.path.join(ROOT, 'assets/game/sheet_candy.png')
OUT_TILES = os.path.join(ROOT, 'assets/game/tiles')
OUT_CHAR = os.path.join(ROOT, 'assets/game/char')
PC = os.path.join(ROOT, 'assets/vendor/kenney/platformer-characters/PNG')
os.makedirs(OUT_TILES, exist_ok=True)
os.makedirs(OUT_CHAR, exist_ok=True)

SRC = Image.open(RL).convert('RGBA')
W, H = SRC.size
px = SRC.load()

# Exact-color candy remap (source RGB -> target RGB).
REMAP = {
    # grass greens -> lemon land
    (0x70, 0xA0, 0x20): (0xF4, 0xDE, 0x70),
    (0x80, 0xB0, 0x30): (0xFB, 0xEA, 0x96),
    (0x60, 0x80, 0x30): (0xDB, 0xBE, 0x4E),
    (0x58, 0x88, 0x20): (0xE2, 0xC9, 0x5B),
    # leaf greens -> pink foliage
    (0x40, 0xA0, 0x60): (0xD8, 0x5A, 0x90),
    (0x50, 0xB0, 0x70): (0xEA, 0x7E, 0xA9),
    (0x20, 0x70, 0x70): (0xB8, 0x3A, 0x6E),
    (0x30, 0x80, 0x70): (0xC4, 0x44, 0x7C),
    (0x40, 0x88, 0x70): (0xD0, 0x50, 0x88),
    # soil/trunks -> caramel + licorice
    (0x80, 0x60, 0x30): (0xC9, 0x6A, 0x3E),
    (0xB0, 0x80, 0x50): (0xE8, 0xA2, 0x66),
    (0x90, 0x70, 0x50): (0xB0, 0x4A, 0x4A),
    (0xA0, 0x78, 0x48): (0xDC, 0x8E, 0x52),
    # stone grays -> lilac castle stone
    (0xC0, 0xC0, 0xC0): (0xE0, 0xC6, 0xE8),
    (0xC0, 0xC0, 0xD0): (0xE0, 0xC6, 0xE8),
    (0xD0, 0xD0, 0xD0): (0xEC, 0xD8, 0xF2),
    (0xB0, 0xB0, 0xC0): (0xCB, 0xAB, 0xD6),
    (0xA8, 0xA8, 0xB8): (0xC4, 0xA2, 0xD0),
    (0x90, 0x90, 0x90): (0xA8, 0x86, 0xB8),
    (0x80, 0x80, 0x80): (0x97, 0x78, 0xA8),
    (0x80, 0x80, 0x90): (0x97, 0x78, 0xA8),
    (0x70, 0x70, 0x70): (0x88, 0x6A, 0x9A),
}
lookup = {}
for (r, g, b), dst in REMAP.items():
    lookup[(r // 8, g // 8, b // 8)] = dst

# --- foliage/ground ramp: candy-yellow land, bubblegum bushes ---------------
# The exact-key REMAP above only catches colours that appear verbatim in the
# source sheet, and it misses most of the greens - which is why the "lemon"
# meadow rendered as flat green grass. This second pass works on HUE so it
# catches every shade the sheet actually uses.
LEMON = [(0xA8, 0x86, 0x2E), (0xE6, 0xC4, 0x4E), (0xF8, 0xEA, 0xA6)]
PINK = [(0x9C, 0x2E, 0x62), (0xDE, 0x64, 0x9C), (0xFF, 0xB8, 0xD2)]
CHERRY = [(0xA8, 0x22, 0x46), (0xF0, 0x5A, 0x7E), (0xFF, 0xB0, 0xC0)]
LILAC = [(0x84, 0x66, 0x98), (0xC4, 0xA2, 0xD0), (0xF2, 0xE8, 0xF8)]


def ramp_sample(ramp, lum):
    """Sample a 3-stop ramp (dark, mid, light) by 0..255 luminance."""
    f = max(0.0, min(1.0, lum / 255.0))
    if f < 0.5:
        t, a, b = f * 2.0, ramp[0], ramp[1]
    else:
        t, a, b = (f - 0.5) * 2.0, ramp[1], ramp[2]
    return tuple(int(a[i] + (b[i] - a[i]) * t) for i in range(3))


def hue_remap(r, g, b, green_ramp):
    """Return a candy-palette colour for saturated greens / reds, else None.

    `green_ramp` decides what the sheet's greens become: lemon for the ground
    family, bubblegum pink for foliage. The same source sheet is therefore mined
    twice, which is how the meadow and the bushes end up reading as different
    materials instead of as one green mass.
    """
    mx, mn = max(r, g, b), min(r, g, b)
    if mx == 0:
        return None
    sat = (mx - mn) / mx
    lum = (r * 299 + g * 587 + b * 114) // 1000
    if g >= r and g >= b and g - max(r, b) >= 10 and sat >= 0.16:
        return ramp_sample(green_ramp, lum)
    # Vivid reds/oranges (berries, licorice) become candy cherry. The
    # saturation and green-taper guards matter: without them this also swallowed
    # soil, turning the dirt tile bright pink.
    if (r >= g and r >= b and r - max(g, b) >= 30 and sat >= 0.62
            and r >= 0x90 and g < r * 0.72 and b < 0x80):
        return ramp_sample(CHERRY, lum)
    return None


def gray_remap(r, g, b):
    """Neutral greys are the castle masonry; push them to lilac stone."""
    mx, mn = max(r, g, b), min(r, g, b)
    if mx == 0:
        return None
    sat = (mx - mn) / mx
    lum = (r * 299 + g * 587 + b * 114) // 1000
    if sat <= 0.14 and 40 <= lum <= 235:
        return ramp_sample(LILAC, lum)
    return None


# base sheet: only the hand-verified exact-colour remap (stone, trunks, soil...)
out = Image.new('RGBA', (W, H), (0, 0, 0, 0))
opx = out.load()
for y in range(H):
    for x in range(W):
        r, g, b, a = px[x, y]
        if a < 40:
            continue
        d = lookup.get((r // 8, g // 8, b // 8))
        if d is None:
            d = gray_remap(r, g, b)
        opx[x, y] = (d if d else (r, g, b)) + (a,)
out.save(OUT_SHEET)


def hue_variant(green_ramp):
    im = out.copy()
    d = im.load()
    hits = 0
    for y in range(H):
        for x in range(W):
            r, g, b, a = d[x, y]
            if a < 40:
                continue
            nr = hue_remap(r, g, b, green_ramp)
            if nr is not None:
                d[x, y] = nr + (a,)
                hits += 1
    return im, hits


sheet_lemon, n_lemon = hue_variant(LEMON)
sheet_pink, n_pink = hue_variant(PINK)
print(f'sheet variants: {n_lemon} px -> lemon ground, {n_pink} px -> pink foliage')


# Which source sheet each named tile is cut from. Foliage gets pink foliage,
# the ground family gets lemon turf, furniture/stone stay on the neutral base.
LEMON_TILES = {'ground', 'ground_alt', 'ground_speck', 'dirt'}
PINK_TILES = {'bush_top', 'bush_mid', 'bush_base', 'flower_a', 'flower_b', 'flower_c'}


def crop_tile(idx, sheet=None):
    src = out if sheet is None else sheet
    x0 = 1 + (idx % 57) * 17
    y0 = 1 + (idx // 57) * 17
    tile = src.crop((x0, y0, x0 + 16, y0 + 16))
    # edge-heal: the source sheet's 1px gutters bleed into tile borders and
    # turn into harsh gridlines at 3x nearest scaling. Replace the outer ring
    # with the adjacent inner pixel so tiles meet seamlessly.
    d = tile.load()
    for i in range(16):
        d[i, 0] = d[i, 1]
        d[i, 15] = d[i, 14]
        d[0, i] = d[1, i]
        d[15, i] = d[14, i]
    return tile


# Named tiles. Indexes verified: 62 flat grass (sample map floor), 5 flat grass
# alt, 63 dirt/caramel, 920/917/919 pink flowers, 400/402 foliage, 923 bush base,
# 761/763 lilac stone, 875 stone floor, 674 grass speck.
NAMES = {
    'ground': 62, 'ground_alt': 5, 'ground_speck': 674,
    'dirt': 63,
    # 920/917/919 are NOT flowers in this sheet - they crop to flat grey blocks
    # that scattered as grey squares across the meadow. They alias the ground
    # tile so the scatter pass is a no-op; the meadow's candy variety comes from
    # the procedural decor in world.rs instead.
    'flower_a': 62, 'flower_b': 5, 'flower_c': 62,
    'bush_top': 400, 'bush_mid': 402, 'bush_base': 923,
    'stone': 761, 'stone_alt': 763, 'floor': 875,
}
# Home interior tiles: furniture band verified visually in the sheet probe
# (bed 85/86/88, table 93/94, door 32, window 40, rug 92, fireplace 13/14,
# shelf 31, wood walls 18/19, floor planks 118).
NAMES.update({
    'bed_head': 85, 'bed_foot': 86, 'bed_wide': 88,
    'table_a': 93, 'table_b': 94,
    'door': 32, 'window': 40, 'rug': 92,
    'fire_lit': 13, 'fire_dark': 14,
    'shelf': 31, 'wood_wall': 18, 'wood_wall_alt': 19,
    # idx 118 was recorded as floor planks but crops to a teal tile; the home
    # floor now reuses the verified wood band instead of a cyan board floor.
    'planks': 19,
})
for name, idx in NAMES.items():
    sheet = sheet_lemon if name in LEMON_TILES else (
        sheet_pink if name in PINK_TILES else out)
    crop_tile(idx, sheet).save(os.path.join(OUT_TILES, name + '.png'))

# ---- candy ground detail -----------------------------------------------------
# The roguelike sheet's flat grass cell is genuinely two tones, so the meadow
# tiled out as one solid lemon slab. These passes keep the CC0 cell as the base
# and add authored pixel detail on top: sugar grain, mottled patches, sprinkled
# candy and real blossom tiles (the sheet has no usable flowers - the indices
# that looked like flowers crop to flat grey blocks).
import random

TILE = 16
SUGAR = (250, 232, 160)
SUGAR_DIM = (214, 182, 78)
MOTTLE = [(226, 194, 88), (244, 216, 112), (232, 202, 96)]
SPRINKLE = [(255, 132, 178), (150, 232, 220), (255, 244, 240), (255, 196, 110)]
STEM = (146, 96, 58)
PETALS = [
    ((255, 138, 176), (255, 196, 216)),
    ((255, 246, 240), (255, 208, 226)),
    ((154, 226, 220), (206, 246, 242)),
]


def open_tile(name):
    return Image.open(os.path.join(OUT_TILES, name + '.png')).convert('RGBA')


def seal(img, base=(235, 206, 102)):
    """Fill fully transparent pixels with the ground base colour. The sheet's
    grass cell carries a few see-through pixels, which punched black holes into
    the meadow wherever that tile was drawn."""
    d = img.load()
    for y in range(TILE):
        for x in range(TILE):
            if d[x, y][3] < 40:
                d[x, y] = base + (255,)
    return img


def sugar_grain(img, rng, grain=0.10, mottle=0.10):
    """Scatter faint two-tone grain so a flat cell reads as a surface."""
    d = img.load()
    for y in range(TILE):
        for x in range(TILE):
            r, g, b, a = d[x, y]
            if a < 40:
                continue
            if rng.random() < mottle:
                m = MOTTLE[rng.randrange(len(MOTTLE))]
                d[x, y] = m + (a,)
            elif rng.random() < grain:
                d[x, y] = SUGAR_DIM + (a,)
            elif rng.random() < grain * 0.6:
                d[x, y] = SUGAR + (a,)
    return img


def sprinkled(img, rng, n=5):
    """Candy sprinkles: 2px dashes of the token hues."""
    d = img.load()
    for _ in range(n):
        col = SPRINKLE[rng.randrange(len(SPRINKLE))]
        x, y = rng.randrange(2, TILE - 2), rng.randrange(2, TILE - 2)
        d[x, y] = col + (255,)
        d[x + 1, y] = col + (255,)
        if rng.random() < 0.4:
            d[x + 1, y + 1] = tuple(int(c * 0.85) for c in col) + (255,)
    return img


for _name, _seed, _spr in (('ground', 11, 0), ('ground_alt', 23, 0), ('ground_speck', 37, 2)):
    _rng = random.Random(_seed)
    _img = sugar_grain(seal(open_tile(_name)), _rng, grain=0.09, mottle=0.08)
    if _spr:
        sprinkled(_img, _rng, _spr)
    _img.save(os.path.join(OUT_TILES, _name + '.png'))


def blossom(idx):
    """A 16x16 candy blossom: caramel stem, two leaves, five round petals."""
    im = Image.new('RGBA', (TILE, TILE), (0, 0, 0, 0))
    d = im.load()
    petal, petal_hi = PETALS[idx % len(PETALS)]
    for y in range(9, TILE):
        d[8, y] = STEM + (255,)
    for (x, y) in ((7, 12), (6, 13), (9, 11), (10, 12)):
        d[x, y] = STEM + (255,)
    for (x, y) in ((6, 12), (10, 12), (7, 13)):
        d[x, y] = (172, 210, 118) + (255,)
    cx, cy = 8, 6
    for (dx, dy) in ((0, -3), (-3, -1), (-2, 2), (2, 2), (3, -1)):
        for ox in (0, 1):
            for oy in (0, 1):
                x, y = cx + dx + ox, cy + dy + oy
                if 0 <= x < TILE and 0 <= y < TILE:
                    d[x, y] = petal + (255,)
    for (dx, dy) in ((-3, -2), (-1, -3), (2, -2)):
        x, y = cx + dx, cy + dy
        if 0 <= x < TILE and 0 <= y < TILE:
            d[x, y] = petal_hi + (255,)
    for (dx, dy) in ((-1, -1), (0, -1), (-1, 0), (0, 0)):
        d[cx + dx, cy + dy] = (255, 224, 130) + (255,)
    return im


for _i, _name in enumerate(('flower_a', 'flower_b', 'flower_c')):
    blossom(_i).save(os.path.join(OUT_TILES, _name + '.png'))
print('candy ground + blossom tiles emitted')

# ---- candy hedges -----------------------------------------------------------
# The roguelike sheet has no bush that reads as foliage: the cells that looked
# like shrubs crop to striped crates, so the Candy Land horizon turned into a
# row of boxes. These three hedge clumps are authored in the token palette
# instead: overlapping mounds in three pinks with sugar speckles on top.
HEDGE = [
    ((0xD8, 0x4A, 0x86), (0xF0, 0x7C, 0xA8), (0xFF, 0xC2, 0xD8)),
    ((0xC8, 0x3F, 0x7A), (0xE6, 0x6E, 0x9C), (0xFF, 0xB4, 0xD0)),
    ((0xE0, 0x5C, 0x94), (0xF4, 0x8C, 0xB4), (0xFF, 0xCC, 0xE2)),
]


def hedge(idx):
    """48x32 bush clump: three mounds, lit from the upper left."""
    from PIL import ImageDraw
    w, h = 48, 32
    im = Image.new('RGBA', (w, h), (0, 0, 0, 0))
    dark, mid, hi = HEDGE[idx % len(HEDGE)]
    mounds = [(13, 22, 12, 9), (24, 18, 13, 11), (35, 23, 11, 8)]
    # base silhouette first, one flat dark pass so the clump reads as one mass
    for (cx, cy, rx, ry) in mounds:
        ImageDraw.Draw(im).ellipse((cx - rx, cy - ry, cx + rx, cy + ry), fill=dark + (255,))
    # fill the seam between mounds so no sky shows through the middle
    ImageDraw.Draw(im).rectangle((10, 16, 40, 30), fill=dark + (255,))
    for (cx, cy, rx, ry) in mounds:
        ImageDraw.Draw(im).ellipse((cx - rx + 2, cy - ry + 1, cx + rx - 2, cy + ry - 3), fill=mid + (255,))
    for (cx, cy, rx, ry) in mounds:
        ImageDraw.Draw(im).ellipse((cx - rx + 5, cy - ry + 2, cx + rx - 12, cy + ry - 12), fill=hi + (255,))
    d = im.load()
    rng = random.Random(90 + idx)
    sugar = [(255, 255, 255), (255, 236, 246), (255, 214, 232)]
    for _ in range(9):
        x, y = rng.randrange(6, 42), rng.randrange(8, 26)
        if d[x, y][3] > 0:
            d[x, y] = sugar[rng.randrange(len(sugar))] + (255,)
    return im


for _i, _name in enumerate(('bush_soft_a', 'bush_soft_b', 'bush_soft_c')):
    hedge(_i).save(os.path.join(OUT_TILES, _name + '.png'))
print('candy hedges emitted')

# alpha-foot contact shadows: soft elliptical shadow baked from the alpha
# bbox of each walk frame so every cast gets an exactly-fitted shadow.
SHADOW_W = 96
SHADOW_H = 24
for cast in ('river', 'jin', 'kandy', 'matt', 'genie', 'guard_a', 'guard_b',
             'nadia', 'grandparent', 'gill', 'scal', 'lady'):
    src = os.path.join(OUT_CHAR, cast + '.png')
    if not os.path.exists(src):
        continue
    img = Image.open(src).convert('RGBA')
    bb = img.getbbox()
    if bb is None:
        continue
    feet_y = bb[3]  # bottom of visible pixels = ground contact
    content_w = bb[2] - bb[0]
    sh = Image.new('RGBA', (SHADOW_W, SHADOW_H), (0, 0, 0, 0))
    sd = sh.load()
    rx = max(10.0, content_w * 0.44)
    ry = 7.0
    cy = SHADOW_H / 2.0
    for yy in range(SHADOW_H):
        for xx in range(SHADOW_W):
            # offset so ellipse center sits at canvas center-x, bottom-ish
            dx = (xx - SHADOW_W / 2.0) / rx
            dy = (yy - cy) / ry
            dd = dx * dx + dy * dy
            if dd < 1.0:
                sd[xx, yy] = (40, 20, 45, int(96 * (1.0 - dd)))
    sh.save(os.path.join(OUT_CHAR, cast + '_shadow.png'))
print('tiles emitted:', len(NAMES), '+ contact shadows')

# ---- animated cast ----------------------------------------------------------
# From Kenney Platformer Characters (CC0): five bodies with real stand/walk
# poses, side view, right-facing. Bodies chosen per cast member and recolored
# to the candy tokens with palette remaps over the real (pre-shaded) colors.
# Every frame is fitted onto a 96x144 canvas (2:3), bottom aligned, so existing
# draw code is untouched.

CANVAS_W, CANVAS_H = 96, 144
FLOOR = CANVAS_H - 2  # content bottom sits 2px above the canvas bottom


def pc_img(body, pose):
    return Image.open(os.path.join(PC, body, 'Poses', '%s_%s.png' % (body.lower(), pose))).convert('RGBA')


def lum(c):
    return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]


def nearest_remap(img, remap, tol=26):
    """Remap pixels within `tol` rgb distance of a remap source key to the
    key's target, preserving the pixel's original luminance (shading)."""
    data = img.load()
    for y in range(img.height):
        for x in range(img.width):
            r, g, b, a = data[x, y]
            if a < 40:
                continue
            best_d, best_src, best_tgt = 1e9, None, None
            for (sr, sg, sb), (tr, tg, tb) in remap:
                d = (r - sr) ** 2 + (g - sg) ** 2 + (b - sb) ** 2
                if d < best_d:
                    best_d, best_src, best_tgt = d, (sr, sg, sb), (tr, tg, tb)
            if best_src is not None and best_d <= tol * tol:
                s = lum(best_src)
                k = lum((r, g, b)) / s if s > 0 else 1.0
                k = min(1.45, max(0.55, k))
                data[x, y] = (
                    min(255, int(best_tgt[0] * k)),
                    min(255, int(best_tgt[1] * k)),
                    min(255, int(best_tgt[2] * k)),
                    a,
                )


def crown(img, gold=(255, 214, 120, 255), plum=(214, 130, 170, 255)):
    """Small pixel crown on top of the head; finds the head by scanning the
    central column band for the first opaque row."""
    data = img.load()
    top = None
    for y in range(img.height):
        for x in range(CANVAS_W // 4, 3 * CANVAS_W // 4):
            if data[x, y][3] > 60:
                top = y
                break
        if top is not None:
            break
    if top is None or top < 3:
        return
    cw, ch = 26, 7
    cx0 = (CANVAS_W - cw) // 2
    cy = max(0, top - ch)
    for i in range(cw):
        zz = 1 if i % 6 in (1, 4) else 0
        col = gold if zz == 0 else plum
        x = cx0 + i
        if 0 <= x < CANVAS_W:
            for dy in range(ch):
                y = cy + dy
                if 0 <= y < CANVAS_H and data[x, y][3] < 60:
                    data[x, y] = col
    # jewel
    jx, jy = CANVAS_W // 2, cy + 3
    for dx in (-1, 0, 1):
        for dy in (-1, 0, 1):
            x, y = jx + dx, jy + dy
            if 0 <= x < CANVAS_W and 0 <= y < CANVAS_H:
                data[x, y] = (255, 60, 120, 255)


def emit_cast(name, body, remap, scale=1.0, add_crown=False, pose_map=None):
    pose_map = pose_map or {'stand': 'stand', 'w1': 'walk1', 'w2': 'walk2'}
    imgs = {key: pc_img(body, pose) for key, pose in pose_map.items()}
    if scale != 1.0:
        for k in imgs:
            w = max(1, int(imgs[k].width * scale))
            h = max(1, int(imgs[k].height * scale))
            imgs[k] = imgs[k].resize((w, h), Image.NEAREST)
    # union bounding box across frames (bottom row = feet)
    boxes = [im.getbbox() for im in imgs.values()]
    ux0 = min(b[0] for b in boxes)
    uy0 = min(b[1] for b in boxes)
    ux1 = max(b[2] for b in boxes)
    uy1 = max(b[3] for b in boxes)
    cw, ch = ux1 - ux0, uy1 - uy0
    if ch > CANVAS_H - 4 or cw > CANVAS_W - 2:
        raise SystemExit('%s content %dx%d exceeds canvas' % (name, cw, ch))
    ox = (CANVAS_W - cw) // 2 - ux0
    oy = FLOOR - uy1
    for key, im in imgs.items():
        canvas = Image.new('RGBA', (CANVAS_W, CANVAS_H), (0, 0, 0, 0))
        canvas.paste(im, (ox, oy), im)
        if remap:
            nearest_remap(canvas, remap)
        if add_crown:
            crown(canvas)
        fn = os.path.join(OUT_CHAR, name if key == 'stand' else '%s_%s' % (name, key))
        canvas.save(fn + '.png')
    print('char', name, '(content %dx%d)' % (cw, ch))


# Remaps are (real source colour -> candy target). Source clusters were read off
# the shipped poses so shading keeps working; luminance is preserved per pixel.
# River: adventurer body, blue-grey coat recolored to river blue.
emit_cast('river', 'Adventurer', [
    ((0x7E, 0x95, 0xA3), (0x2E, 0x86, 0xE8)),
    ((0x63, 0x74, 0x80), (0x20, 0x68, 0xB8)),
])
# Jin: player body, green shirt -> gingerbread brown (blue jeans stay).
emit_cast('jin', 'Player', [
    ((0x2E, 0xCC, 0x71), (0xC8, 0x82, 0x3C)),
    ((0x25, 0xA8, 0x5C), (0xA0, 0x68, 0x2A)),
])
# Guards: soldier bodies, uniform recolored plum vs slate (face/boots stay).
emit_cast('guard_a', 'Soldier', [
    ((0x54, 0x4B, 0x4B), (0xB0, 0x60, 0xA8)),
    ((0x5D, 0x53, 0x53), (0x9A, 0x52, 0x94)),
    ((0x48, 0x48, 0x48), (0x8E, 0x4A, 0x84)),
    ((0x95, 0x75, 0x44), (0xC0, 0x6A, 0x9C)),
    ((0x88, 0x91, 0x76), (0xA4, 0x6E, 0x90)),
    ((0x74, 0x7B, 0x64), (0x96, 0x62, 0x84)),
])
emit_cast('guard_b', 'Soldier', [
    ((0x54, 0x4B, 0x4B), (0x5A, 0x7A, 0xB0)),
    ((0x5D, 0x53, 0x53), (0x50, 0x6C, 0x9A)),
    ((0x48, 0x48, 0x48), (0x46, 0x62, 0x8E)),
    ((0x95, 0x75, 0x44), (0x66, 0x84, 0xC0)),
    ((0x88, 0x91, 0x76), (0x5E, 0x78, 0xA4)),
    ((0x74, 0x7B, 0x64), (0x54, 0x6E, 0x96)),
])
# Kandy Kween: female body, blue dress -> hot pink, plus crown.
emit_cast('kandy', 'Female', [
    ((0x34, 0x98, 0xDB), (0xE8, 0x4E, 0x9E)),
    ((0x2E, 0x88, 0xC4), (0xC9, 0x3C, 0x86)),
], add_crown=True)
# Matt: player body dressed as an attendant in white with slate trousers.
emit_cast('matt', 'Player', [
    ((0x2E, 0xCC, 0x71), (0xEC, 0xF0, 0xF4)),
    ((0x25, 0xA8, 0x5C), (0xC4, 0xCE, 0xD8)),
    ((0x34, 0x98, 0xDB), (0x5D, 0x73, 0x86)),
    ((0x2A, 0x7D, 0xB4), (0x4A, 0x61, 0x76)),
    ((0x2F, 0x8A, 0xC7), (0x52, 0x69, 0x7C)),
    ((0x55, 0x92, 0xB7), (0x5D, 0x73, 0x86)),
    ((0x4D, 0x7E, 0x9A), (0x4A, 0x61, 0x76)),
])
# Nadia: female body scaled to a child, blue dress -> soft mauve.
emit_cast('nadia', 'Female', [
    ((0x34, 0x98, 0xDB), (0xF0, 0xA0, 0xC0)),
    ((0x2E, 0x88, 0xC4), (0xD0, 0x86, 0xAA)),
], scale=0.72)
# Aquarius citizens: same bodies, aquatic palette so they read as underwater folk.
# Gillbert (adventurer, turquoise coat), Scalleon (soldier, deep navy + seafoam),
# Market Lady (female, coral dress).
emit_cast('gill', 'Adventurer', [
    ((0x7E, 0x95, 0xA3), (0x35, 0xB8, 0xA2)),
    ((0x63, 0x74, 0x80), (0x26, 0x8F, 0x7E)),
])
emit_cast('scal', 'Soldier', [
    ((0x54, 0x4B, 0x4B), (0x3A, 0x6F, 0xAE)),
    ((0x5D, 0x53, 0x53), (0x32, 0x61, 0x9A)),
    ((0x48, 0x48, 0x48), (0x2E, 0x58, 0x8C)),
    ((0x95, 0x75, 0x44), (0x4A, 0x9A, 0x9E)),
    ((0x88, 0x91, 0x76), (0x56, 0x8C, 0x96)),
    ((0x74, 0x7B, 0x64), (0x4C, 0x7C, 0x86)),
])
emit_cast('lady', 'Female', [
    ((0x34, 0x98, 0xDB), (0xF2, 0x7F, 0x8E)),
    ((0x2E, 0x88, 0xC4), (0xD4, 0x6A, 0x7C)),
])
# Grandparent: adventurer body, coat -> warm tan, hair/pants -> gray.
emit_cast('grandparent', 'Adventurer', [
    ((0x7E, 0x95, 0xA3), (0xC4, 0xA0, 0x74)),
    ((0x63, 0x74, 0x80), (0xA1, 0x82, 0x58)),
    ((0x54, 0x4B, 0x4B), (0x84, 0x8C, 0x98)),
    ((0x5D, 0x54, 0x54), (0x72, 0x7A, 0x88)),
])

# ---- aquarius fauna & flora (Kenney Fish Pack, CC0) -----------------------
# Tight-crop the fish / seaweed / rock sprites so the engine can draw them at
# native content size with a clean bottom anchor.
FISH = os.path.join(ROOT, 'assets/vendor/kenney/fish-pack/PNG/Default')
OUT_AQUA = os.path.join(ROOT, 'assets/game/aqua')
os.makedirs(OUT_AQUA, exist_ok=True)

AQUA = {
    'fish': ['fish_blue', 'fish_brown', 'fish_green', 'fish_grey_long_a',
             'fish_grey_long_b', 'fish_grey', 'fish_orange', 'fish_pink', 'fish_red'],
    'weed': ['seaweed_grass_a', 'seaweed_grass_b', 'seaweed_green_b',
             'seaweed_green_d', 'seaweed_pink_a', 'seaweed_pink_c'],
    'rock': ['rock_a', 'rock_b'],
    'bg': ['background_rock_a', 'background_rock_b', 'background_seaweed_b'],
}
for kind, names in AQUA.items():
    for n in names:
        src = Image.open(os.path.join(FISH, n + '.png')).convert('RGBA')
        bb = src.getbbox()
        if bb is None:
            print('skip', n, '(empty)')
            continue
        pad = 1
        box = (max(0, bb[0] - pad), max(0, bb[1] - pad),
               min(src.width, bb[2] + pad), min(src.height, bb[3] + pad))
        src.crop(box).save(os.path.join(OUT_AQUA, n + '.png'))
print('aqua emitted:', sum(len(v) for v in AQUA.values()))

# ---- cosma: neon void assets (Kenney Space Kit + Planets, CC0) -----------
# The Space Kit's Side rocks/crystals/meteors all share one baked Mars-brown
# ramp, so a single luminance-preserving remap per cast turns them into neon
# void-crystal floats (teal / magenta / violet) while keeping every shade.
# The walkway reuses the roguelike sheet's stone cells through a second exact
# remap (deep indigo), the same way the candy land re-ramped them lilac.
OUT_COSMA = os.path.join(ROOT, 'assets/game/cosma')
os.makedirs(OUT_COSMA, exist_ok=True)

SPACE_SIDE = os.path.join(ROOT, 'assets/vendor/kenney/space-kit/Side')
PLANET_SRC = os.path.join(ROOT, 'assets/vendor/kenney/planets/PNG')


def cosma_mars_remap(img, keys):
    """Remap the Space Kit Mars ramp: (src, tgt) key pairs, luminance kept."""
    remap = [
        ((0x96, 0x5A, 0x46), keys[0]),
        ((0xB4, 0x64, 0x50), keys[1]),
        ((0xD2, 0x78, 0x5A), keys[2]),
        ((0xF0, 0x8C, 0x6E), keys[3]),
        ((0xFA, 0x96, 0x6E), keys[4]),
    ]
    nearest_remap(img, remap, tol=34)


def tight_crop_save(img, name, out_dir):
    bb = img.getbbox()
    if bb is None:
        print('skip', name, '(empty)')
        return
    pad = 2
    box = (max(0, bb[0] - pad), max(0, bb[1] - pad),
           min(img.width, bb[2] + pad), min(img.height, bb[3] + pad))
    img.crop(box).save(os.path.join(out_dir, name + '.png'))


def emit_cosma_float(src, name, keys):
    im = Image.open(os.path.join(SPACE_SIDE, src)).convert('RGBA')
    cosma_mars_remap(im, keys)
    tight_crop_save(im, name, OUT_COSMA)


# void-crystal casts: violet (portal stone), teal, magenta.
VIOLET = ((0x2B, 0x1F, 0x56), (0x43, 0x32, 0x8A), (0x6A, 0x54, 0xC0), (0x94, 0x78, 0xE8), (0xAC, 0x94, 0xFF))
TEAL = ((0x0F, 0x3D, 0x4D), (0x17, 0x8E, 0xA6), (0x3F, 0xC8, 0xDC), (0x8E, 0xF2, 0xFF), (0xD8, 0xFF, 0xFF))
PINK = ((0x57, 0x10, 0x3F), (0xA5, 0x2A, 0x78), (0xE0, 0x4A, 0xA8), (0xFF, 0x86, 0xD0), (0xFF, 0xD2, 0xEF))
DEEP = ((0x22, 0x18, 0x44), (0x35, 0x28, 0x6A), (0x50, 0x3F, 0x96), (0x74, 0x5C, 0xC4), (0x92, 0x7A, 0xE8))

emit_cosma_float('rock_crystalsLargeA.png', 'crystal_teal', TEAL)
emit_cosma_float('rock_crystalsLargeB.png', 'crystal_pink', PINK)
emit_cosma_float('rock_crystals.png', 'crystal_violet', VIOLET)
emit_cosma_float('rock_largeA.png', 'boulder_a', DEEP)
emit_cosma_float('rock_largeB.png', 'boulder_b', DEEP)
emit_cosma_float('meteor_detailed.png', 'meteor_a', DEEP)
emit_cosma_float('meteor.png', 'meteor_b', DEEP)
emit_cosma_float('rocks_smallA.png', 'pebble_cyan', ((0x0C, 0x2E, 0x40), (0x14, 0x78, 0x94), (0x2E, 0xB8, 0xD4), (0x6E, 0xE8, 0xFF), (0xC0, 0xFF, 0xFF)))
emit_cosma_float('rocks_smallB.png', 'pebble_pink', ((0x4A, 0x0C, 0x34), (0x8C, 0x20, 0x62), (0xC8, 0x3C, 0x90), (0xF8, 0x78, 0xC4), (0xFF, 0xC8, 0xE8)))

# -- void castle-stone floor tiles (exact remap of roguelike grays) ---------
# Same source cells as the candy stone but through an indigo ramp.
GRAY_SRC = {
    0xC0C0C0: (0x6E, 0x5C, 0xB8),
    0xC0C0D0: (0x6E, 0x5C, 0xB8),
    0xD0D0D0: (0x84, 0x72, 0xD0),
    0xB0B0C0: (0x5C, 0x4A, 0xA0),
    0xB0B0B0: (0x5C, 0x4A, 0xA0),
    0xA8A8B8: (0x4A, 0x3A, 0x86),
    0xA8A8A8: (0x4A, 0x3A, 0x86),
    0x909090: (0x37, 0x2A, 0x68),
    0x808090: (0x2C, 0x22, 0x55),
    0x808080: (0x2C, 0x22, 0x55),
    0x707070: (0x24, 0x1C, 0x46),
    0xCCCCCC: (0x7C, 0x6A, 0xC8),
    0xD8D8D8: (0x8E, 0x7C, 0xD8),
}


def crop_src_tile(idx):
    x0 = 1 + (idx % 57) * 17
    y0 = 1 + (idx // 57) * 17
    return SRC.crop((x0, y0, x0 + 16, y0 + 16))


def remap_tile_exact(img):
    """Exact-ish remap: quantize grays into 8-wide buckets (like the candy
    sheet pass) so the sheet's near-grays all hit a ramp entry."""
    img = img.convert('RGBA')
    data = img.load()
    buckets = {}
    for packed, d in GRAY_SRC.items():
        r, g, b = (packed >> 16) & 0xFF, (packed >> 8) & 0xFF, packed & 0xFF
        buckets[(r // 8, g // 8, b // 8)] = d
    for y in range(img.height):
        for x in range(img.width):
            r, g, b, a = data[x, y]
            if a < 40:
                continue
            d = buckets.get((r // 8, g // 8, b // 8))
            if d:
                data[x, y] = d + (a,)
    return img


# floor body: three subtly different blocks (a, b + darker speck)
void_a = remap_tile_exact(crop_src_tile(761))
void_b = remap_tile_exact(crop_src_tile(763))
void_c = remap_tile_exact(crop_src_tile(875))
void_a.save(os.path.join(OUT_COSMA, 'floor_a.png'))
void_b.save(os.path.join(OUT_COSMA, 'floor_b.png'))
void_c.save(os.path.join(OUT_COSMA, 'floor_c.png'))


# glowing rim tile: bright top facets + scattered white glints (lit ledge edge)
rim = void_a.copy()
rd = rim.load()
for x in range(16):
    for y in range(5):
        r, g, b, a = rd[x, y]
        f = 1.0 - y * 0.13
        rd[x, y] = (min(255, int(r + (190 - r) * f)), min(255, int(g + (150 - g) * f)), min(255, int(b + (255 - b) * f)), a)
for x in range(16):
    if (x * 7) % 5 == 0 or (x * 11) % 7 == 0:
        gl = [(255, 214, 255), (255, 255, 255), (190, 255, 250)][x % 3]
        rd[x, 0] = gl + (255,)
        if x + 3 < 16:
            rd[x + 3, 2] = tuple(min(255, int(cc * 0.85)) for cc in gl) + (255,)
rim.save(os.path.join(OUT_COSMA, 'floor_rim.png'))


# a couple of floor cells with baked neon veins for sparse accent
for vein_name, vein_col in (('vein_pink', (255, 90, 170)), ('vein_cyan', (110, 230, 255))):
    v = void_a.copy()
    vd = v.load()
    for x in range(16):
        for y in range(10, 14):
            r, g, b, a = vd[x, y]
            vd[x, y] = tuple(min(255, int(cc * 0.5 + vc * 0.5)) for cc, vc in zip((r, g, b), vein_col)) + (a,)
    vd[7, 9] = vein_col + (255,)
    vd[8, 10] = vein_col + (255,)
    vd[9, 11] = vein_col + (255,)
    vd[10, 12] = vein_col + (255,)
    v.save(os.path.join(OUT_COSMA, vein_name + '.png'))
print('cosma tiles emitted')

# planets (tight-cropped, already downscaled at vendor time)
for f in sorted(os.listdir(PLANET_SRC)):
    if not f.endswith('.png'):
        continue
    im = Image.open(os.path.join(PLANET_SRC, f)).convert('RGBA')
    tight_crop_save(im, f[:-4], OUT_COSMA)
print('planets emitted')

# ---- cosma floor v2: OpenGameArt castle masonry (CC0) + Neon Node trim -----
# The roguelike-derived tiles above read as generic stone; the OGA castle
# tileset is purpose-drawn castle brickwork (CC0, direct download), so the
# walkway body now comes from it, recolored through the same indigo ramp.
# Neon Node (also CC0, OGA) supplies real neon line-art 24x24 cells that are
# recolored to the token hues and inlaid as glowing trim blocks.
OGA_DIR = os.path.join(ROOT, 'assets/vendor/oga')
castle = Image.open(os.path.join(OGA_DIR, 'castle-tileset.png')).convert('RGBA')
neonode = Image.open(os.path.join(OGA_DIR, 'neonode.png')).convert('RGBA')


def castle_cell(cx, cy):
    return castle.crop((cx * 16, cy * 16, cx * 16 + 16, cy * 16 + 16))


# hue ramp for the masonry (same family as GRAY_SRC, tuned warmer stone)
INDIGO = [
    (0x8E, 0x7C, 0xD8), (0x7C, 0x6A, 0xC8), (0x6E, 0x5C, 0xB8),
    (0x5C, 0x4A, 0xA0), (0x4A, 0x3A, 0x86), (0x37, 0x2A, 0x68),
    (0x2C, 0x22, 0x55), (0x24, 0x1C, 0x46), (0x1A, 0x14, 0x34),
]


def luminance_key(px):
    r, g, b, a = px
    if a < 40:
        return None
    return (r + g + b) // 3


def remap_by_luma(img, ramp):
    """Map luminance through a fixed indigo ramp, hue-normalized. Keeps every
    shade of the masonry mortar/brick texture while forcing the token hue."""
    im = img.copy()
    d = im.load()
    for y in range(im.height):
        for x in range(im.width):
            px = d[x, y]
            if px[3] < 40:
                continue
            lum = (px[0] + px[1] + px[2]) // 3
            # ramp index by luminance band (0..255 -> ramp)
            idx = min(len(ramp) - 1, lum * len(ramp) // 256)
            d[x, y] = ramp[idx] + (px[3],)
    return im


# brick cells: (0,0)-(2,2) large brick faces, mortar shading variance
# (0,3)-(2,3) are solid dark fills -> edge/shadow variants
floor_v2_a = remap_by_luma(castle_cell(0, 0), INDIGO)
floor_v2_b = remap_by_luma(castle_cell(1, 1), INDIGO)
floor_v2_c = remap_by_luma(castle_cell(4, 0), INDIGO)
floor_v2_a.save(os.path.join(OUT_COSMA, 'floor_a.png'))
floor_v2_b.save(os.path.join(OUT_COSMA, 'floor_b.png'))
floor_v2_c.save(os.path.join(OUT_COSMA, 'floor_c.png'))
print('cosma floor v2 (OGA castle masonry) emitted')

# neon line-art nodes: recolor dark cells to the void base, keep the neon
# line hues but normalize toward token cyan/magenta/violet
NEON_BG = (0x14, 0x10, 0x2C)
NEON_HUES = {
    (4, 244, 255): (110, 230, 255),     # cyan stays cyan
    (255, 4, 187): (255, 90, 170),      # magenta -> token pink
    (86, 255, 4): (150, 110, 255),      # green -> violet
    (246, 255, 4): (255, 214, 120),     # yellow -> gold accent
    (60, 87, 85): (70, 60, 120),        # teal-steel -> dim slate
    (87, 85, 60): (60, 50, 100),        # olive -> dim slate
    (17, 17, 17): NEON_BG,
    (34, 34, 34): (0x1E, 0x18, 0x3C),
}


def remap_neon(img):
    im = img.copy()
    d = im.load()
    for y in range(im.height):
        for x in range(im.width):
            px = d[x, y]
            if px[3] < 40:
                continue
            best, bd = None, 10**9
            for (sr, sg, sb), tgt in NEON_HUES.items():
                dd = (px[0]-sr)**2 + (px[1]-sg)**2 + (px[2]-sb)**2
                if dd < bd:
                    bd, best = dd, tgt
            d[x, y] = best + (px[3],)
    return im


for c in range(5):
    cell = neonode.crop((c * 24, 0, c * 24 + 24, 24))
    remap_neon(cell).save(os.path.join(OUT_COSMA, f'neon_node_{c}.png'))
print('neon nodes emitted (OGA Neon Node, CC0)')

print('done -> assets/game/{sheet_candy.png,tiles/,char/,aqua/,cosma/}')


# ---- fx: Kenney Particle Pack (CC0) ----------------------------------------
# Soft-alpha particle sprites (512px masters) downscaled to 128px: they are all
# smooth gradients/glows so the downscale loses nothing visible at 1x-3x and
# keeps the browser payload small. These replace the hand-drawn circles the
# particle system used to emit, which is the single biggest "reads as
# programmer art" tell in the overworld.
OUT_FX = os.path.join(ROOT, 'assets/game/fx')
os.makedirs(OUT_FX, exist_ok=True)
PART_SRC = os.path.join(ROOT, 'assets/vendor/kenney/particle-pack/PNG (Transparent)')

# emitted name -> source file stem
FX_PARTICLES = {
    # rising dust / silt / foam / fog
    'smoke_1': 'smoke_02', 'smoke_2': 'smoke_05', 'smoke_3': 'smoke_08',
    # hot glints, meteors, sparkle cores
    'spark_1': 'spark_01', 'spark_2': 'spark_04', 'spark_3': 'spark_06',
    # four-point stars: candy twinkle, catchable star, rune glint
    'star_1': 'star_02', 'star_2': 'star_05', 'star_3': 'star_07', 'star_4': 'star_09',
    # rune / genie magic
    'magic_1': 'magic_01', 'magic_2': 'magic_03', 'magic_3': 'magic_05',
    'fire_1': 'fire_01', 'fire_2': 'fire_02',
    'flame_1': 'flame_02', 'flame_2': 'flame_04', 'flame_3': 'flame_06',
    # shockwave rings / ripples / bubbles
    'circle_1': 'circle_02', 'circle_2': 'circle_03', 'circle_3': 'circle_05',
    # speed streaks / star trails
    'trace_1': 'trace_02', 'trace_2': 'trace_05', 'trace_3': 'trace_07',
    'twirl_1': 'twirl_01', 'twirl_2': 'twirl_03',
    # soft light pools / god-ray sprites
    'light_1': 'light_01', 'light_2': 'light_02', 'light_3': 'light_03',
    'flare': 'flare_01',
    # kicked-up earth
    'dirt_1': 'dirt_01', 'dirt_2': 'dirt_03',
}
FX_SIZE = 128
for out_name, src_stem in FX_PARTICLES.items():
    src_path = os.path.join(PART_SRC, src_stem + '.png')
    im = Image.open(src_path).convert('RGBA').resize((FX_SIZE, FX_SIZE), Image.LANCZOS)
    im.save(os.path.join(OUT_FX, out_name + '.png'))
print(f'fx particles emitted: {len(FX_PARTICLES)} -> assets/game/fx/')

# ---- real CC0 sound effects (Kenney: interface / impact / rpg audio) -------
# Replaces the synthesized sine blips with mastered, layered samples. Gameplay
# reads as a game the moment its footsteps and confirms stop being sine waves.
import shutil
OUT_AUDIO = os.path.join(ROOT, 'assets/game/audio')
os.makedirs(OUT_AUDIO, exist_ok=True)
IFACE = os.path.join(ROOT, 'assets/vendor/kenney/interface-sounds/Audio')
IMPACT = os.path.join(ROOT, 'assets/vendor/kenney/impact-sounds/Audio')
RPG = os.path.join(ROOT, 'assets/vendor/kenney/rpg-audio/Audio')

# Only what `src/audio/mod.rs` actually loads: every extra file is another
# request in the browser before the first frame draws.
SFX_FILES = {
    'blip':        (IFACE, 'tick_001.ogg'),
    'good':        (IFACE, 'confirmation_002.ogg'),
    'bad':         (IFACE, 'error_004.ogg'),
    'page':        (RPG,   'bookFlip1.ogg'),
    'splash':      (IMPACT, 'impactSoft_heavy_000.ogg'),
    'click':       (IFACE, 'click_003.ogg'),
    'back':        (IFACE, 'back_002.ogg'),
    'select':      (IFACE, 'select_001.ogg'),
    'coin':        (RPG,   'handleCoins.ogg'),
    'question':    (IFACE, 'question_001.ogg'),
    'step_grass_a':  (IMPACT, 'footstep_grass_000.ogg'),
    'step_grass_b':  (IMPACT, 'footstep_grass_003.ogg'),
    'step_sand_a':   (IMPACT, 'footstep_carpet_001.ogg'),
    'step_sand_b':   (IMPACT, 'footstep_carpet_003.ogg'),
    'step_stone_a':  (IMPACT, 'footstep_concrete_000.ogg'),
    'step_stone_b':  (IMPACT, 'footstep_concrete_002.ogg'),
    'step_wood_a':   (IMPACT, 'footstep_wood_001.ogg'),
    'step_wood_b':   (IMPACT, 'footstep_wood_003.ogg'),
    'step_snow_a':   (IMPACT, 'footstep_snow_001.ogg'),
    'step_snow_b':   (IMPACT, 'footstep_snow_004.ogg'),
}
missing = []
for out_name, (folder, fname) in SFX_FILES.items():
    src = os.path.join(folder, fname)
    if not os.path.exists(src):
        missing.append(fname)
        continue
    shutil.copyfile(src, os.path.join(OUT_AUDIO, out_name + '.ogg'))
print(f'sfx emitted: {len(SFX_FILES) - len(missing)} -> assets/game/audio/')
if missing:
    print('  MISSING:', missing)
