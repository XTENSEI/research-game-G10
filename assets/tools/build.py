#!/usr/bin/env python3
"""Deterministic CC0 + pixel-art asset pipeline for Gift of Education.

Generates all runtime textures and audio under `assets/game/` so the game runs
natively and on wasm32 without Python at runtime.
"""
import os
import random
import shutil
from PIL import Image

import pixel_cast

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
VENDOR = os.path.join(ROOT, "assets/vendor")
GAME = os.path.join(ROOT, "assets/game")

OUT_CHAR = os.path.join(GAME, "char")
OUT_PORTRAIT = os.path.join(GAME, "portraits")
OUT_INTRO = os.path.join(GAME, "intro")
OUT_PROPS = os.path.join(GAME, "props")
OUT_TILES = os.path.join(GAME, "tiles")
OUT_AQUA = os.path.join(GAME, "aqua")
OUT_COSMA = os.path.join(GAME, "cosma")
OUT_UI = os.path.join(GAME, "ui")
OUT_FX = os.path.join(GAME, "fx")
OUT_AUDIO = os.path.join(GAME, "audio")

for d in (OUT_CHAR, OUT_PORTRAIT, OUT_INTRO, OUT_PROPS, OUT_TILES, OUT_AQUA, OUT_COSMA, OUT_UI, OUT_FX, OUT_AUDIO):
    os.makedirs(d, exist_ok=True)

# ---------------------------------------------------------------------------
# 1. Bespoke Pixel-Art Cast, Bust Portraits, Intro Plates, & Biome Props
# ---------------------------------------------------------------------------
for stale in ("jin", "kandy", "matt"):
    for suf in ("", "_w1", "_w2", "_shadow"):
        p = os.path.join(OUT_CHAR, f"{stale}{suf}.png")
        if os.path.exists(p):
            os.remove(p)

pixel_cast.generate_all(OUT_CHAR, OUT_PORTRAIT, OUT_INTRO, OUT_PROPS)

# ---------------------------------------------------------------------------
# 2. Crisp 16x16 Biome Tiles (Kelp Forest, Home, Coral City, Crab Castle)
# ---------------------------------------------------------------------------
TT_TILES = os.path.join(VENDOR, "kenney/tiny-town/Tiles")
TD_TILES = os.path.join(VENDOR, "kenney/tiny-dungeon/Tiles")
PP_TILES = os.path.join(VENDOR, "kenney/pixel-platformer/Tiles")


def load_tt(idx):
    return Image.open(os.path.join(TT_TILES, f"tile_{idx:04d}.png")).convert("RGBA")


def load_td(idx):
    return Image.open(os.path.join(TD_TILES, f"tile_{idx:04d}.png")).convert("RGBA")


def load_pp(idx):
    return Image.open(os.path.join(PP_TILES, f"tile_{idx:04d}.png")).convert("RGBA")


def ramp_tile(im, dark, mid, light):
    """Luminance-preserving 3-stop palette tint for pixel tiles."""
    out = im.copy()
    px = out.load()
    for y in range(out.height):
        for x in range(out.width):
            r, g, b, a = px[x, y]
            if a < 40:
                continue
            f = (r * 0.299 + g * 0.587 + b * 0.114) / 255.0
            if f < 0.5:
                t, p, q = f * 2.0, dark, mid
            else:
                t, p, q = (f - 0.5) * 2.0, mid, light
            px[x, y] = (
                int(p[0] + (q[0] - p[0]) * t),
                int(p[1] + (q[1] - p[1]) * t),
                int(p[2] + (q[2] - p[2]) * t),
                a,
            )
    return out


# Kelp Forest lush underwater moss/seabed turf + sandy path + castle stone
kelp_turf_a = ramp_tile(load_tt(0), (36, 108, 86), (62, 168, 124), (116, 224, 164))
kelp_turf_b = ramp_tile(load_tt(1), (36, 108, 86), (56, 158, 118), (110, 214, 156))
kelp_flower = ramp_tile(load_tt(2), (38, 112, 92), (82, 186, 146), (255, 226, 146))
kelp_sand = ramp_tile(load_td(49), (146, 132, 92), (196, 178, 124), (236, 220, 166))
coral_sand = ramp_tile(load_td(49), (132, 122, 88), (184, 168, 120), (222, 206, 156))
castle_stone = ramp_tile(load_td(14), (72, 46, 76), (118, 72, 112), (176, 116, 152))
castle_stone_alt = ramp_tile(load_td(15), (66, 42, 70), (108, 64, 104), (162, 106, 140))

kelp_turf_a.save(os.path.join(OUT_TILES, "ground.png"))
kelp_turf_b.save(os.path.join(OUT_TILES, "ground_alt.png"))
kelp_flower.save(os.path.join(OUT_TILES, "ground_speck.png"))
kelp_sand.save(os.path.join(OUT_TILES, "dirt.png"))
coral_sand.save(os.path.join(OUT_TILES, "coral_sand.png"))
castle_stone.save(os.path.join(OUT_TILES, "stone.png"))
castle_stone_alt.save(os.path.join(OUT_TILES, "stone_alt.png"))

# Home interior crisp 16x16 plank floor, wainscot wall, window, door, rug
def make_home_tiles():
    # Plank floor
    planks = Image.new("RGBA", (16, 16), (138, 92, 58, 255))
    p = planks.load()
    for y in range(16):
        for x in range(16):
            if y in (0, 8):
                p[x, y] = (88, 56, 36, 255)
            elif (y < 8 and x == 8) or (y > 8 and x == 2):
                p[x, y] = (96, 62, 40, 255)
            elif y in (1, 9):
                p[x, y] = (158, 110, 72, 255)
            elif (x * 7 + y * 3) % 13 == 0:
                p[x, y] = (126, 82, 50, 255)
    planks.save(os.path.join(OUT_TILES, "planks.png"))

    # Warm cream-plum wallpaper with wood wainscot strip
    wall = Image.new("RGBA", (16, 16), (82, 54, 78, 255))
    w = wall.load()
    for y in range(16):
        for x in range(16):
            if x % 4 == 0:
                w[x, y] = (72, 46, 68, 255)
            elif x % 4 == 2:
                w[x, y] = (92, 62, 86, 255)
    wall.save(os.path.join(OUT_TILES, "wood_wall.png"))

    wall_trim = wall.copy()
    wt = wall_trim.load()
    for x in range(16):
        for y in range(12, 16):
            wt[x, y] = (116, 76, 52, 255) if y > 12 else (148, 100, 68, 255)
    wall_trim.save(os.path.join(OUT_TILES, "wood_wall_alt.png"))

    # Window with starry night sky
    win = Image.new("RGBA", (16, 16), (118, 76, 48, 255))
    wd = win.load()
    for y in range(2, 14):
        for x in range(2, 14):
            wd[x, y] = (24, 30, 68, 255)
    for i in range(16):
        wd[i, 7] = (118, 76, 48, 255)
        wd[7, i] = (118, 76, 48, 255)
    wd[4, 4] = (250, 232, 162, 255)
    wd[11, 10] = (220, 236, 255, 255)
    win.save(os.path.join(OUT_TILES, "window.png"))

    # Cozy woven rug
    rug = Image.new("RGBA", (16, 16), (168, 72, 114, 255))
    rg = rug.load()
    for y in range(16):
        for x in range(16):
            if x in (0, 15) or y in (0, 15):
                rg[x, y] = (232, 186, 112, 255)
            elif 3 <= x <= 12 and 3 <= y <= 12 and (x == 3 or x == 12 or y == 3 or y == 12):
                rg[x, y] = (214, 120, 156, 255)
    rug.save(os.path.join(OUT_TILES, "rug.png"))

    load_tt(85).save(os.path.join(OUT_TILES, "door.png"))


make_home_tiles()

# ---------------------------------------------------------------------------
# 3. Aquarius / Coral City & Ansimuz Underwater Pack (CC0)
# ---------------------------------------------------------------------------
UW_PNG = os.path.join(VENDOR, "oga/underwater/underwater-diving-files/PNG")
PROPS_IMG = Image.open(os.path.join(UW_PNG, "environment/props.png")).convert("RGBA")
TILES_IMG = Image.open(os.path.join(UW_PNG, "environment/tiles.png")).convert("RGBA")

# Verified crop boxes from Ansimuz props.png
UW_CROPS = {
    "neptune_statue": (432, 41, 524, 272),
    "neptune_coral": (596, 41, 716, 272),
    "coral_arch": (177, 34, 375, 239),
    "ruin_hall": (752, 70, 997, 256),
    "sea_bush": (434, 332, 480, 383),
    "coral_red": (514, 327, 562, 383),
    "sea_kelp": (614, 317, 666, 384),
}
for name, box in UW_CROPS.items():
    PROPS_IMG.crop(box).save(os.path.join(OUT_AQUA, f"{name}.png"))

# Ansimuz seamless ocean backdrop & mid reef
shutil.copyfile(os.path.join(UW_PNG, "environment/background.png"), os.path.join(OUT_AQUA, "ansimuz_bg.png"))
shutil.copyfile(os.path.join(UW_PNG, "environment/midground.png"), os.path.join(OUT_AQUA, "ansimuz_mid.png"))
# Seabed edge tile strip from Ansimuz tiles.png
TILES_IMG.crop((16, 144, 160, 192)).save(os.path.join(OUT_AQUA, "seabed_ledge.png"))

# Animated fish sprites from Ansimuz
for src_name, dst_name in (("fish.png", "anim_fish"), ("fish-big.png", "anim_fish_big"), ("fish-dart.png", "anim_fish_dart")):
    shutil.copyfile(os.path.join(UW_PNG, "enemies", src_name), os.path.join(OUT_AQUA, f"{dst_name}.png"))

# Kenney Fish Pack tight-cropped fish & coral/weed
FISH = os.path.join(VENDOR, "kenney/fish-pack/PNG/Default")
KENNEY_AQUA = {
    "fish": [
        "fish_blue", "fish_brown", "fish_green", "fish_grey_long_a",
        "fish_grey_long_b", "fish_grey", "fish_orange", "fish_pink", "fish_red",
    ],
    "weed": [
        "seaweed_grass_a", "seaweed_grass_b", "seaweed_green_b",
        "seaweed_green_d", "seaweed_pink_a", "seaweed_pink_c",
    ],
    "rock": ["rock_a", "rock_b"],
    "bg": ["background_rock_a", "background_rock_b", "background_seaweed_b"],
}
for _, names in KENNEY_AQUA.items():
    for n in names:
        src = Image.open(os.path.join(FISH, n + ".png")).convert("RGBA")
        bb = src.getbbox()
        if bb is None:
            continue
        box = (max(0, bb[0] - 1), max(0, bb[1] - 1), min(src.width, bb[2] + 1), min(src.height, bb[3] + 1))
        src.crop(box).save(os.path.join(OUT_AQUA, n + ".png"))

# ---------------------------------------------------------------------------
# 4. Somber Slums / Cosma Assets (OGA Castle + Neon Node + Space Kit + Planets)
# ---------------------------------------------------------------------------
OGA_DIR = os.path.join(VENDOR, "oga")
SPACE_SIDE = os.path.join(VENDOR, "kenney/space-kit/Side")
PLANET_SRC = os.path.join(VENDOR, "kenney/planets/PNG")

castle = Image.open(os.path.join(OGA_DIR, "castle-tileset.png")).convert("RGBA")
neonode = Image.open(os.path.join(OGA_DIR, "neonode.png")).convert("RGBA")

INDIGO = [
    (0x8E, 0x7C, 0xD8), (0x7C, 0x6A, 0xC8), (0x6E, 0x5C, 0xB8),
    (0x5C, 0x4A, 0xA0), (0x4A, 0x3A, 0x86), (0x37, 0x2A, 0x68),
    (0x2C, 0x22, 0x55), (0x24, 0x1C, 0x46), (0x1A, 0x14, 0x34),
]


def remap_luma(img, ramp):
    im = img.copy()
    d = im.load()
    for y in range(im.height):
        for x in range(im.width):
            px = d[x, y]
            if px[3] < 40:
                continue
            l = (px[0] + px[1] + px[2]) // 3
            idx = min(len(ramp) - 1, l * len(ramp) // 256)
            d[x, y] = ramp[idx] + (px[3],)
    return im


def castle_cell(cx, cy):
    return castle.crop((cx * 16, cy * 16, cx * 16 + 16, cy * 16 + 16))


fa = remap_luma(castle_cell(0, 0), INDIGO)
fb = remap_luma(castle_cell(1, 1), INDIGO)
fc = remap_luma(castle_cell(4, 0), INDIGO)
fa.save(os.path.join(OUT_COSMA, "floor_a.png"))
fb.save(os.path.join(OUT_COSMA, "floor_b.png"))
fc.save(os.path.join(OUT_COSMA, "floor_c.png"))

rim = fa.copy()
rd = rim.load()
for x in range(16):
    for y in range(4):
        r, g, b, a = rd[x, y]
        f = 1.0 - y * 0.2
        rd[x, y] = (min(255, int(r + (170 - r) * f)), min(255, int(g + (230 - g) * f)), min(255, int(b + (255 - b) * f)), a)
rim.save(os.path.join(OUT_COSMA, "floor_rim.png"))

for vname, vc in (("vein_pink", (255, 90, 170)), ("vein_cyan", (110, 230, 255))):
    v = fa.copy()
    vd = v.load()
    for x in range(6, 12):
        vd[x, x] = vc + (255,)
    v.save(os.path.join(OUT_COSMA, f"{vname}.png"))

NEON_HUES = {
    (4, 244, 255): (110, 230, 255),
    (255, 4, 187): (255, 90, 170),
    (86, 255, 4): (150, 110, 255),
    (246, 255, 4): (255, 214, 120),
    (60, 87, 85): (70, 60, 120),
    (87, 85, 60): (60, 50, 100),
    (17, 17, 17): (0x14, 0x10, 0x2C),
    (34, 34, 34): (0x1E, 0x18, 0x3C),
}
for cidx in range(5):
    cell = neonode.crop((cidx * 24, 0, cidx * 24 + 24, 24))
    d = cell.load()
    for y in range(24):
        for x in range(24):
            px = d[x, y]
            if px[3] < 40:
                continue
            best = min(NEON_HUES.items(), key=lambda kv: (px[0]-kv[0][0])**2 + (px[1]-kv[0][1])**2 + (px[2]-kv[0][2])**2)[1]
            d[x, y] = best + (px[3],)
    cell.save(os.path.join(OUT_COSMA, f"neon_node_{cidx}.png"))


def lum(c):
    return 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2]


def cosma_float(src, name, keys):
    im = Image.open(os.path.join(SPACE_SIDE, src)).convert("RGBA")
    remap = [
        ((0x96, 0x5A, 0x46), keys[0]),
        ((0xB4, 0x64, 0x50), keys[1]),
        ((0xD2, 0x78, 0x5A), keys[2]),
        ((0xF0, 0x8C, 0x6E), keys[3]),
        ((0xFA, 0x96, 0x6E), keys[4]),
    ]
    d = im.load()
    for y in range(im.height):
        for x in range(im.width):
            r, g, b, a = d[x, y]
            if a < 40:
                continue
            best_s, best_t, bd = None, None, 1e9
            for s, t in remap:
                dd = (r-s[0])**2 + (g-s[1])**2 + (b-s[2])**2
                if dd < bd:
                    bd, best_s, best_t = dd, s, t
            if bd <= 36*36:
                s0 = lum(best_s)
                k = min(1.45, max(0.55, lum((r, g, b)) / s0 if s0 > 0 else 1.0))
                d[x, y] = (min(255, int(best_t[0]*k)), min(255, int(best_t[1]*k)), min(255, int(best_t[2]*k)), a)
    bb = im.getbbox()
    if bb:
        im.crop((max(0, bb[0]-2), max(0, bb[1]-2), min(im.width, bb[2]+2), min(im.height, bb[3]+2))).save(os.path.join(OUT_COSMA, f"{name}.png"))


VIOLET = ((0x2B, 0x1F, 0x56), (0x43, 0x32, 0x8A), (0x6A, 0x54, 0xC0), (0x94, 0x78, 0xE8), (0xAC, 0x94, 0xFF))
TEAL = ((0x0F, 0x3D, 0x4D), (0x17, 0x8E, 0xA6), (0x3F, 0xC8, 0xDC), (0x8E, 0xF2, 0xFF), (0xD8, 0xFF, 0xFF))
PINK = ((0x57, 0x10, 0x3F), (0xA5, 0x2A, 0x78), (0xE0, 0x4A, 0xA8), (0xFF, 0x86, 0xD0), (0xFF, 0xD2, 0xEF))
DEEP = ((0x22, 0x18, 0x44), (0x35, 0x28, 0x6A), (0x50, 0x3F, 0x96), (0x74, 0x5C, 0xC4), (0x92, 0x7A, 0xE8))

cosma_float("rock_crystalsLargeA.png", "crystal_teal", TEAL)
cosma_float("rock_crystalsLargeB.png", "crystal_pink", PINK)
cosma_float("rock_crystals.png", "crystal_violet", VIOLET)
cosma_float("rock_largeA.png", "boulder_a", DEEP)
cosma_float("rock_largeB.png", "boulder_b", DEEP)
cosma_float("meteor_detailed.png", "meteor_a", DEEP)
cosma_float("meteor.png", "meteor_b", DEEP)
cosma_float("rocks_smallA.png", "pebble_cyan", TEAL)
cosma_float("rocks_smallB.png", "pebble_pink", PINK)

for f in sorted(os.listdir(PLANET_SRC)):
    if f.endswith(".png"):
        im = Image.open(os.path.join(PLANET_SRC, f)).convert("RGBA")
        bb = im.getbbox()
        if bb:
            im.crop(bb).save(os.path.join(OUT_COSMA, f))

# ---------------------------------------------------------------------------
# 5. UI 9-Slice Borders, Vials, Hearts, Crisp Pixel Stars
# ---------------------------------------------------------------------------
UI_BORDER = os.path.join(VENDOR, "kenney/fantasy-ui-borders/PNG/Default")
shutil.copyfile(os.path.join(UI_BORDER, "Panel/panel-005.png"), os.path.join(OUT_UI, "panel_fill.png"))
shutil.copyfile(os.path.join(UI_BORDER, "Border/panel-border-015.png"), os.path.join(OUT_UI, "panel_border.png"))
shutil.copyfile(os.path.join(UI_BORDER, "Divider/divider-001.png"), os.path.join(OUT_UI, "divider.png"))

load_td(125).save(os.path.join(OUT_UI, "vial_empty.png"))
load_td(126).save(os.path.join(OUT_UI, "vial_green.png"))
load_td(127).save(os.path.join(OUT_UI, "vial_red.png"))
load_td(128).save(os.path.join(OUT_UI, "vial_blue.png"))
load_pp(44).save(os.path.join(OUT_UI, "heart_full.png"))
load_pp(46).save(os.path.join(OUT_UI, "heart_empty.png"))
load_pp(67).save(os.path.join(OUT_UI, "gem.png"))


def make_star_sprite(dying):
    im = Image.new("RGBA", (16, 16), (0, 0, 0, 0))
    px = im.load()
    if dying:
        edge, body, core = (130, 28, 48, 255), (235, 62, 74, 255), (255, 174, 152, 255)
    else:
        edge, body, core = (214, 148, 44, 255), (255, 224, 96, 255), (255, 252, 232, 255)
    for y in range(16):
        for x in range(16):
            dx = abs(x - 7.5)
            dy = abs(y - 7.5)
            d = (dx ** 0.72) + (dy ** 0.72)
            if d <= 2.3:
                px[x, y] = core
            elif d <= 3.6:
                px[x, y] = body
            elif d <= 4.3:
                px[x, y] = edge
    return im


make_star_sprite(False).save(os.path.join(OUT_UI, "star.png"))
make_star_sprite(True).save(os.path.join(OUT_UI, "dying_star.png"))

# ---------------------------------------------------------------------------
# 6. FX Particles (Kenney Particle Pack, CC0)
# ---------------------------------------------------------------------------
PART_SRC = os.path.join(VENDOR, "kenney/particle-pack/PNG (Transparent)")
FX_PARTICLES = {
    "smoke_1": "smoke_02", "smoke_2": "smoke_05", "smoke_3": "smoke_08",
    "spark_1": "spark_01", "spark_2": "spark_04", "spark_3": "spark_06",
    "star_1": "star_02", "star_2": "star_05", "star_3": "star_07", "star_4": "star_09",
    "magic_1": "magic_01", "magic_2": "magic_03", "magic_3": "magic_05",
    "fire_1": "fire_01", "fire_2": "fire_02",
    "flame_1": "flame_02", "flame_2": "flame_04", "flame_3": "flame_06",
    "circle_1": "circle_02", "circle_2": "circle_03", "circle_3": "circle_05",
    "trace_1": "trace_02", "trace_2": "trace_05", "trace_3": "trace_07",
    "twirl_1": "twirl_01", "twirl_2": "twirl_03",
    "light_1": "light_01", "light_2": "light_02", "light_3": "light_03",
    "flare": "flare_01",
    "dirt_1": "dirt_01", "dirt_2": "dirt_03",
}
for out_name, src_stem in FX_PARTICLES.items():
    im = Image.open(os.path.join(PART_SRC, src_stem + ".png")).convert("RGBA").resize((128, 128), Image.LANCZOS)
    im.save(os.path.join(OUT_FX, out_name + ".png"))

# ---------------------------------------------------------------------------
# 7. Audio: SFX (Kenney CC0) + Music Loops (OpenGameArt CC0)
# ---------------------------------------------------------------------------
IFACE = os.path.join(VENDOR, "kenney/interface-sounds/Audio")
IMPACT = os.path.join(VENDOR, "kenney/impact-sounds/Audio")
RPG = os.path.join(VENDOR, "kenney/rpg-audio/Audio")
MUSIC = os.path.join(VENDOR, "oga/music")
UW_SND = os.path.join(VENDOR, "oga/underwater/underwater-diving-files/Sound")

AUDIO_FILES = {
    "blip": (IFACE, "tick_001.ogg"),
    "good": (IFACE, "confirmation_002.ogg"),
    "bad": (IFACE, "error_004.ogg"),
    "page": (RPG, "bookFlip1.ogg"),
    "splash": (IMPACT, "impactSoft_heavy_000.ogg"),
    "click": (IFACE, "click_003.ogg"),
    "back": (IFACE, "back_002.ogg"),
    "select": (IFACE, "select_001.ogg"),
    "coin": (RPG, "handleCoins.ogg"),
    "question": (IFACE, "question_001.ogg"),
    "step_grass_a": (IMPACT, "footstep_grass_000.ogg"),
    "step_grass_b": (IMPACT, "footstep_grass_003.ogg"),
    "step_sand_a": (IMPACT, "footstep_carpet_001.ogg"),
    "step_sand_b": (IMPACT, "footstep_carpet_003.ogg"),
    "step_stone_a": (IMPACT, "footstep_concrete_000.ogg"),
    "step_stone_b": (IMPACT, "footstep_concrete_002.ogg"),
    "step_wood_a": (IMPACT, "footstep_wood_001.ogg"),
    "step_wood_b": (IMPACT, "footstep_wood_003.ogg"),
    "step_snow_a": (IMPACT, "footstep_snow_001.ogg"),
    "step_snow_b": (IMPACT, "footstep_snow_004.ogg"),
    # CC0 Music tracks
    "mus_town": (MUSIC, "town_theme.ogg"),
    "mus_kelp": (MUSIC, "magic_town.ogg"),
    "mus_coral": (MUSIC, "eyes_of_the_ocean.ogg"),
    "mus_cave": (UW_SND, "watery_cave_loop.ogg"),
    "mus_slums": (MUSIC, "dark_eyes_of_the_ocean.ogg"),
    "mus_rainbow": (MUSIC, "iridescent_deep.ogg"),
    "mus_battle": (MUSIC, "8bit_battle.ogg"),
}
for out_name, (folder, fname) in AUDIO_FILES.items():
    shutil.copyfile(os.path.join(folder, fname), os.path.join(OUT_AUDIO, out_name + ".ogg"))

print("build.py finished cleanly!")
