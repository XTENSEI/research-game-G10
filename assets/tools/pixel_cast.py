#!/usr/bin/env python3
"""Hand-crafted pixel-art sprites, expressive dialogue portraits, biome props,
and Undertale-style intro plates for Gift of Education.

All characters match SCRIPT RES.pdf (pages 9-14) on clean 32x36 (or 48x40 for
wide creatures like Tow-Tou / Queen Crab) pixel grids with hue-shifted outlines,
3-tone cel shading, and distinct silhouettes. Portraits are 48x48 bust views.
"""
import math
import os
from PIL import Image, ImageDraw

OUTLINE_WARM = (34, 20, 44, 255)
OUTLINE_SEA = (14, 26, 46, 255)
OUTLINE_ABYSS = (10, 12, 26, 255)
CLEAR = (0, 0, 0, 0)


def shade(col, factor):
    r, g, b = col[:3]
    a = col[3] if len(col) > 3 else 255
    return (
        max(0, min(255, int(r * factor))),
        max(0, min(255, int(g * factor))),
        max(0, min(255, int(b * factor))),
        a,
    )


def outline_image(im, outline=OUTLINE_SEA):
    """Add crisp 1px pixel-art silhouette outline around non-transparent pixels."""
    w, h = im.size
    src = im.load()
    out = Image.new("RGBA", (w, h), CLEAR)
    dst = out.load()
    for y in range(h):
        for x in range(w):
            if src[x, y][3] >= 128:
                dst[x, y] = src[x, y]
                continue
            hit = False
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = x + dx, y + dy
                if 0 <= nx < w and 0 <= ny < h and src[nx, ny][3] >= 128:
                    hit = True
                    break
            if hit:
                dst[x, y] = outline
    return out


class Canvas:
    def __init__(self, w, h):
        self.w = w
        self.h = h
        self.im = Image.new("RGBA", (w, h), CLEAR)
        self.d = ImageDraw.Draw(self.im)

    def px(self, x, y, col):
        ix, iy = int(round(x)), int(round(y))
        if 0 <= ix < self.w and 0 <= iy < self.h:
            self.im.putpixel((ix, iy), col if len(col) == 4 else col + (255,))

    def rect(self, x, y, w, h, col):
        if w <= 0 or h <= 0:
            return
        self.d.rectangle((x, y, x + w - 1, y + h - 1), fill=col if len(col) == 4 else col + (255,))

    def oval(self, cx, cy, rx, ry, col):
        self.d.ellipse((cx - rx, cy - ry, cx + rx, cy + ry), fill=col if len(col) == 4 else col + (255,))

    def poly(self, pts, col):
        self.d.polygon(pts, fill=col if len(col) == 4 else col + (255,))

    def line(self, x0, y0, x1, y1, col, width=1):
        self.d.line((x0, y0, x1, y1), fill=col if len(col) == 4 else col + (255,), width=width)

    def eyes(self, lx, rx, y, iris=(36, 28, 56), mood="normal", wide=False):
        if mood == "closed" or mood == "sick":
            self.rect(lx, y + 1, 3, 1, (48, 32, 60))
            self.rect(rx, y + 1, 3, 1, (48, 32, 60))
            if mood == "sick":
                self.px(lx - 1, y + 2, (230, 140, 160))
                self.px(rx + 3, y + 2, (230, 140, 160))
            return
        eh = 3 if wide else 2
        self.rect(lx, y, 2, eh, (250, 252, 255))
        self.rect(rx, y, 2, eh, (250, 252, 255))
        self.rect(lx + 1, y + (eh - 2), 1, 2, iris)
        self.rect(rx + 1, y + (eh - 2), 1, 2, iris)
        self.px(lx, y, (255, 255, 255))
        self.px(rx, y, (255, 255, 255))
        if mood == "angry":
            self.line(lx - 1, y - 2, lx + 2, y - 1, (48, 22, 34))
            self.line(rx + 2, y - 2, rx - 1, y - 1, (48, 22, 34))
        elif mood == "shocked":
            self.rect(lx, y - 2, 2, 1, (48, 28, 52))
            self.rect(rx, y - 2, 2, 1, (48, 28, 52))

    def finish(self, outline=OUTLINE_SEA):
        if outline is None:
            return self.im
        return outline_image(self.im, outline)


def make_shadow(sprite):
    """Baked elliptical contact shadow matched to sprite foot width."""
    w, h = sprite.size
    bb = sprite.getbbox()
    cw = (bb[2] - bb[0]) if bb else w // 2
    sh = Image.new("RGBA", (w, 12), CLEAR)
    sd = sh.load()
    rx = max(6.0, cw * 0.42)
    ry = 4.2
    cx, cy = w / 2.0, 6.0
    for yy in range(12):
        for xx in range(w):
            dx = (xx - cx) / rx
            dy = (yy - cy) / ry
            d = dx * dx + dy * dy
            if d < 1.0:
                sd[xx, yy] = (18, 12, 32, int(115 * (1.0 - d)))
    return sh


# ---------------------------------------------------------------------------
# Overworld Cast Sprites (32x36 standard, 44x40 for wide bosses/turtles)
# ---------------------------------------------------------------------------

def draw_humanoid(cfg, pose="stand"):
    """Draw clean 32x36 humanoid sprite (River, Nadia, Grandparent, Mark, Jack, Jill, Anglers, Thaumiel, Tenebris)."""
    W, H = 32, 36
    c = Canvas(W, H)
    step = 0 if pose == "stand" else (2 if pose == "w1" else -2)
    bob = 0 if pose == "stand" else -1
    short = cfg.get("short", 0)
    skin = cfg["skin"]
    skin_lo = shade(skin, 0.84)
    shirt = cfg["shirt"]
    shirt_hi = shade(shirt, 1.18)
    shirt_lo = shade(shirt, 0.76)
    pants = cfg["pants"]
    pants_lo = shade(pants, 0.75)
    hair = cfg["hair"]
    hair_hi = shade(hair, 1.2)
    hair_lo = shade(hair, 0.74)

    base_y = H - 2
    # Legs
    ly_l = base_y - 9 + (1 if pose == "w1" else 0)
    ly_r = base_y - 9 + (1 if pose == "w2" else 0)
    c.rect(11 + step // 2, ly_l, 4, 7, pants)
    c.rect(11 + step // 2, ly_l + 5, 4, 2, cfg.get("shoes", (72, 50, 40)))
    c.rect(17 - step // 2, ly_r, 4, 7, pants_lo)
    c.rect(17 - step // 2, ly_r + 5, 4, 2, cfg.get("shoes", (72, 50, 40)))

    # Torso / Coat / Dress
    ty = 15 + bob + short
    if cfg.get("trenchcoat"):
        c.rect(9, ty, 14, 15 - short // 2, shirt)
        c.rect(10, ty, 5, 14 - short // 2, shirt_hi)
        c.rect(19, ty + 1, 4, 13 - short // 2, shirt_lo)
        c.rect(11, ty + 6, 10, 2, cfg.get("trim", (96, 198, 220)))
    else:
        c.rect(10, ty, 12, 11 - short // 2, shirt)
        c.rect(11, ty, 5, 9 - short // 2, shirt_hi)
        c.rect(18, ty + 2, 4, 9 - short // 2, shirt_lo)
        if cfg.get("collar"):
            c.rect(13, ty, 6, 2, cfg["collar"])
        if cfg.get("tie"):
            c.rect(15, ty + 2, 2, 6, cfg["tie"])
        if cfg.get("floral"):
            c.px(13, ty + 4, (255, 196, 98))
            c.px(14, ty + 5, (255, 120, 170))
            c.px(17, ty + 4, (136, 232, 160))

    # Arms (swinging opposite legs)
    c.rect(7, ty + 1 - step // 2, 3, 7, shirt_lo)
    c.rect(7, ty + 7 - step // 2, 3, 2, skin_lo)
    c.rect(22, ty + 1 + step // 2, 3, 7, shirt)
    c.rect(22, ty + 7 + step // 2, 3, 2, skin)

    # Head
    hy = 4 + bob + short
    c.oval(16, hy + 6, 6, 6, skin)
    c.rect(11, hy + 9, 10, 3, skin_lo)
    # Eyes
    eye_col = cfg.get("eye", (40, 28, 60))
    if cfg.get("sunglasses"):
        c.rect(11, hy + 5, 4, 3, (18, 22, 40))
        c.rect(17, hy + 5, 4, 3, (18, 22, 40))
        c.rect(15, hy + 6, 2, 1, (100, 220, 255))
        c.px(12, hy + 5, (110, 240, 255))
        c.px(18, hy + 5, (110, 240, 255))
    else:
        c.eyes(12, 18, hy + 5, iris=eye_col)

    # Species / Role overlays
    species = cfg.get("species", "human")
    if species == "human":
        # Hair cap + bangs
        c.oval(16, hy + 2, 7, 4, hair)
        c.rect(10, hy + 1, 5, 2, hair_hi)
        c.rect(10, hy + 3, 3, 4, hair_lo)
        c.rect(19, hy + 3, 3, 4, hair)
        if cfg.get("long_hair"):
            c.rect(9, hy + 5, 3, 7, hair_lo)
            c.rect(20, hy + 5, 3, 7, hair)
        if cfg.get("beard"):
            c.rect(12, hy + 9, 8, 4, hair)
        if cfg.get("glasses"):
            c.rect(11, hy + 5, 4, 2, (230, 240, 255))
            c.rect(17, hy + 5, 4, 2, (230, 240, 255))
            c.px(15, hy + 5, (180, 150, 90))
    elif species == "billfish":
        # Swordfish sail crest + pointed rostrum protruding right
        c.poly([(11, hy + 2), (16, hy - 3), (20, hy + 2)], cfg["fin"])
        c.poly([(13, hy + 1), (16, hy - 2), (18, hy + 1)], shade(cfg["fin"], 1.25))
        c.poly([(21, hy + 6), (30, hy + 7), (21, hy + 9)], shade(skin, 1.15))
        if cfg.get("monocle"):
            c.rect(17, hy + 4, 4, 4, (235, 210, 120))
            c.rect(18, hy + 5, 2, 2, (160, 240, 255))
    elif species == "angelfish":
        # Tall silver diamond crest + powdered aristocrat wig curls
        c.poly([(10, hy + 2), (16, hy - 3), (22, hy + 2)], (235, 244, 255))
        c.rect(9, hy + 2, 3, 8, (240, 244, 252))
        c.rect(20, hy + 2, 3, 8, (220, 228, 242))
        c.rect(13, ty, 6, 4, (255, 255, 255))  # ruffled cravat
    elif species == "stonefish":
        # Bumpy dorsal ridge + backward cap
        c.rect(10, hy + 1, 12, 3, (190, 62, 74))
        c.rect(7, hy + 2, 4, 2, (150, 44, 56))
        c.px(13, hy + 8, (112, 92, 64))
        c.px(19, hy + 9, (112, 92, 64))
    elif species == "angler":
        # Glowing bioluminescent lure arching forward over brow
        lure = cfg["lure"]
        c.line(15, hy + 1, 19, hy - 3, (56, 70, 96))
        c.line(19, hy - 3, 24, hy + 1, (56, 70, 96))
        c.oval(24, hy + 2, 2, 2, lure)
        c.px(24, hy + 1, (255, 255, 255))
        if cfg.get("hat"):
            c.rect(8, hy + 1, 16, 2, (22, 20, 34))
            c.rect(11, hy - 2, 10, 3, (34, 30, 50))
        elif cfg.get("hood"):
            c.oval(16, hy + 2, 7, 4, cfg["hood"])
    elif species == "reef":
        # Gillbert/Scalleon/Lady aquatic folk fins or coral shawl
        c.poly([(9, hy + 3), (6, hy + 6), (9, hy + 9)], cfg["fin"])
        c.poly([(22, hy + 3), (25, hy + 6), (22, hy + 9)], cfg["fin"])
        if cfg.get("helmet"):
            c.oval(16, hy + 1, 6, 3, (124, 152, 186))
        elif cfg.get("shawl"):
            c.rect(9, ty, 14, 5, cfg["shawl"])
            # basket of sea berries
            c.rect(22, ty + 5, 6, 5, (176, 126, 72))
            c.px(24, ty + 4, (220, 74, 160))
            c.px(26, ty + 4, (140, 210, 255))

    return c.finish(cfg.get("outline", OUTLINE_SEA))


def draw_river_swim(pose="swim1"):
    """River swimming horizontally through underwater currents (36x32)."""
    c = Canvas(36, 32)
    kick = 2 if pose == "swim1" else -2
    # Legs kicking
    c.rect(3, 15 + kick // 2, 9, 4, (182, 137, 82))
    c.rect(4, 19 - kick // 2, 8, 4, (150, 110, 64))
    c.rect(1, 15 + kick // 2, 3, 4, (72, 50, 40))
    c.rect(2, 19 - kick // 2, 3, 4, (72, 50, 40))
    # Blue shirt torso
    c.rect(11, 14, 11, 9, (46, 134, 232))
    c.rect(12, 14, 9, 3, (88, 172, 250))
    # Forward arm reaching
    c.rect(18, 18 + kick // 2, 7, 3, (34, 102, 184))
    c.rect(24, 18 + kick // 2, 3, 3, (242, 196, 160))
    # Head
    c.oval(26, 14, 5, 5, (242, 196, 160))
    c.oval(24, 10, 5, 3, (107, 68, 46))
    c.rect(21, 11, 4, 4, (78, 48, 32))
    c.eyes(25, 28, 13, iris=(40, 28, 60))
    return c.finish(OUTLINE_SEA)


def draw_kenny(pose="stand"):
    """Kenny: Red-orange Kelp Crab with kelp slingbag & scuttling legs (36x36)."""
    c = Canvas(36, 36)
    step = 0 if pose == "stand" else (2 if pose == "w1" else -2)
    bob = 0 if pose == "stand" else -1
    shell = (224, 84, 42)
    shell_hi = (255, 138, 76)
    shell_lo = (168, 52, 24)
    kelp = (74, 168, 92)
    kelp_lo = (46, 118, 64)

    # 6 jointed crab legs
    for i, lx in enumerate((8, 11, 14, 21, 24, 27)):
        off = step if (i % 2 == 0) else -step
        c.rect(lx, 25 + off // 2, 2, 8, shell_lo)
        c.rect(lx + (1 if lx > 18 else -1), 30 + off // 2, 2, 3, shell)

    # Wide oval carapace
    cy = 18 + bob
    c.oval(18, cy, 10, 8, shell)
    c.oval(16, cy - 3, 7, 4, shell_hi)
    c.rect(10, cy + 4, 16, 4, shell_lo)

    # Kelp slingbag strap + side satchel!
    c.line(11, cy - 5, 25, cy + 5, kelp, width=3)
    c.line(11, cy - 6, 24, cy + 4, (112, 206, 120), width=1)
    c.rect(22, cy + 2, 6, 6, kelp_lo)
    c.rect(23, cy + 2, 4, 3, kelp)

    # Raised claws
    c.oval(6, cy - 4 - step // 2, 4, 4, shell_hi)
    c.px(6, cy - 7 - step // 2, CLEAR)
    c.oval(30, cy - 4 + step // 2, 4, 4, shell_hi)

    # Eyestalks & friendly eyes + smile
    c.rect(13, cy - 11, 2, 5, shell)
    c.rect(21, cy - 11, 2, 5, shell)
    c.eyes(13, 21, cy - 12, iris=(28, 20, 38), wide=True)
    c.line(15, cy + 1, 21, cy + 1, (70, 24, 16))
    return c.finish(OUTLINE_WARM)


def draw_seahorse_guard(variant=0, pose="stand"):
    """Seahorse guard with Spartan helmet & crimson-ribboned spear (32x36)."""
    c = Canvas(32, 36)
    bob = 0 if pose == "stand" else (1 if pose == "w1" else -1)
    body = (76, 194, 168) if variant == 0 else (92, 126, 232)
    body_hi = shade(body, 1.2)
    body_lo = shade(body, 0.72)
    armor = (218, 178, 86) if variant == 0 else (186, 196, 218)

    # Spear held vertically
    sx = 25
    c.rect(sx, 2 + bob, 2, 31, (132, 86, 52))
    c.poly([(sx - 1, 5 + bob), (sx + 1, 0), (sx + 3, 5 + bob)], (220, 232, 246))
    c.rect(sx + 2, 6 + bob, 3, 3, (218, 52, 92))  # red ribbon

    # Curled tail + upright seahorse torso
    c.oval(14, 28 + bob, 5, 4, body_lo)
    c.rect(13, 27 + bob, 2, 2, CLEAR)
    c.oval(15, 20 + bob, 5, 7, body)
    c.rect(13, 16 + bob, 3, 8, body_hi)
    # Dorsal fin
    c.poly([(10, 16 + bob), (6, 20 + bob), (10, 24 + bob)], body_hi)

    # Head + seahorse snout
    c.oval(16, 10 + bob, 5, 4, body)
    c.rect(20, 10 + bob, 5, 2, body_hi)
    c.eyes(15, 18, 9 + bob, iris=(24, 24, 48))

    # Spartan helmet + red plume
    c.rect(11, 5 + bob, 10, 4, armor)
    c.poly([(12, 4 + bob), (16, 1 + bob), (20, 4 + bob)], (216, 52, 88))
    return c.finish(OUTLINE_SEA)


def draw_queen_crab(pose="stand"):
    """Queen Crab: Spiky King Crab monarch in royal mantle & crown (44x40)."""
    c = Canvas(44, 40)
    bob = 0 if pose == "stand" else (1 if pose == "w1" else -1)
    shell = (214, 65, 40)
    shell_hi = (250, 114, 72)
    shell_lo = (146, 30, 22)
    cape = (118, 22, 56)

    # Royal crimson cape
    c.poly([(10, 16), (34, 16), (38, 37), (6, 37)], cape)
    c.rect(8, 35, 28, 3, (245, 238, 232))  # ermine trim

    # Spiky carapace & legs
    for lx in (9, 14, 30, 35):
        c.rect(lx, 31 + bob, 3, 7, shell_lo)
    c.oval(22, 20 + bob, 12, 10, shell)
    c.oval(20, 16 + bob, 8, 5, shell_hi)
    # Spikes around shell
    for sx, sy in ((10, 14), (13, 10), (31, 10), (34, 14), (9, 22), (35, 22)):
        c.poly([(sx - 2, sy + 2), (sx, sy - 3), (sx + 2, sy + 2)], shell_lo)

    # Huge royal claws
    c.oval(6, 20 - bob, 5, 5, shell_hi)
    c.oval(38, 20 + bob, 5, 5, shell_hi)

    # Elixir Pearl Necklace!
    for nx in range(15, 30, 3):
        c.rect(nx, 27 + bob, 2, 2, (246, 240, 210))
    c.oval(22, 29 + bob, 2, 2, (110, 244, 255))

    # Golden jewel crown
    c.rect(15, 6 + bob, 14, 4, (244, 198, 72))
    for cx in (15, 21, 27):
        c.rect(cx, 3 + bob, 2, 3, (244, 198, 72))
    c.px(22, 7 + bob, (255, 70, 140))

    c.eyes(17, 25, 15 + bob, iris=(40, 12, 20), mood="angry")
    return c.finish(OUTLINE_WARM)


def draw_sophie(pose="stand"):
    """Sophie: Clownfish Jester (32x36)."""
    c = Canvas(32, 36)
    bob = 0 if pose == "stand" else -1
    orange = (255, 122, 42)
    orange_lo = (204, 78, 20)
    white = (250, 246, 240)

    c.rect(11, 26, 4, 8, (46, 36, 68))
    c.rect(17, 26, 4, 8, (46, 36, 68))
    c.rect(10, 15 + bob, 12, 12, orange)
    c.rect(14, 15 + bob, 4, 12, white)
    c.rect(8, 16 + bob, 3, 7, orange_lo)
    c.rect(21, 16 + bob, 3, 7, orange)
    # Ruffled jester collar
    c.rect(9, 14 + bob, 14, 3, (244, 212, 92))
    # Clownfish head
    c.oval(16, 9 + bob, 6, 5, orange)
    c.rect(19, 6 + bob, 2, 7, white)
    c.eyes(12, 18, 8 + bob, iris=(28, 22, 44))
    # Jester 2-prong cap with bells
    c.poly([(11, 6 + bob), (6, 1 + bob), (15, 4 + bob)], (216, 48, 86))
    c.poly([(21, 6 + bob), (26, 1 + bob), (17, 4 + bob)], (82, 194, 130))
    c.rect(5, 1 + bob, 2, 2, (250, 216, 86))
    c.rect(25, 1 + bob, 2, 2, (250, 216, 86))
    return c.finish(OUTLINE_WARM)


def draw_towtou(pose="stand"):
    """Tow-Tou: Leatherback sea turtle with cart on back, beret, beard, scar, prosthetic fin (48x40)."""
    c = Canvas(48, 40)
    bob = 0 if pose == "stand" else (1 if pose == "w1" else -1)
    shell = (42, 74, 80)
    shell_hi = (66, 108, 114)
    ridge = (146, 184, 176)
    skin = (72, 118, 108)

    # Passenger cart strapped on top of shell
    c.rect(10, 4 + bob, 18, 11, (146, 96, 58))
    c.rect(11, 5 + bob, 16, 8, (178, 124, 78))
    c.rect(9, 4 + bob, 20, 2, (216, 180, 96))  # brass rail
    c.line(10, 15 + bob, 28, 22 + bob, (92, 62, 42), width=2)

    # Large leatherback carapace
    c.oval(20, 23 + bob, 14, 9, shell)
    for ry in (18, 22, 26):
        c.line(9, ry + bob, 31, ry + bob, ridge)
    c.oval(18, 18 + bob, 10, 3, shell_hi)

    # Back flipper + Brass prosthetic front flipper!
    c.poly([(8, 28 + bob), (3, 35 - bob), (13, 34 - bob)], skin)
    c.poly([(28, 27 + bob), (34, 37 + bob), (24, 34 + bob)], (220, 178, 76))  # brass prosthetic fin
    c.rect(28, 27 + bob, 4, 3, (150, 110, 46))

    # Turtle head + white beard + red beret + eye scar
    c.oval(37, 20 + bob, 6, 5, skin)
    c.rect(35, 23 + bob, 6, 6, (238, 240, 245))  # wise beard
    c.eyes(35, 40, 18 + bob, iris=(24, 24, 40))
    c.line(34, 16 + bob, 37, 21 + bob, (198, 62, 76))  # eye scar
    c.oval(36, 14 + bob, 6, 2, (196, 44, 72))  # maroon beret
    return c.finish(OUTLINE_SEA)


def draw_genie(pose="stand"):
    """Magic Fire Genie: blazing flame spirit rising from glowing vortex (36x40)."""
    c = Canvas(36, 40)
    sway = 0 if pose == "stand" else (2 if pose == "w1" else -2)
    fire_out = (242, 92, 34)
    fire_mid = (255, 168, 54)
    fire_core = (255, 240, 142)

    # Swirling flame vortex tail
    c.poly([(18 + sway, 38), (11, 26), (25, 26)], fire_out)
    c.poly([(18 + sway // 2, 35), (14, 25), (22, 25)], fire_mid)
    # Torso & broad shoulders
    c.poly([(8, 16), (28, 16), (23, 27), (13, 27)], fire_out)
    c.poly([(11, 17), (25, 17), (21, 26), (15, 26)], fire_mid)
    c.rect(12, 24, 12, 2, (238, 198, 86))  # golden belt
    # Arms + bracers
    c.rect(5, 15 - sway // 2, 4, 8, fire_mid)
    c.rect(27, 15 + sway // 2, 4, 8, fire_mid)
    c.rect(5, 21 - sway // 2, 4, 2, (238, 198, 86))
    c.rect(27, 21 + sway // 2, 4, 2, (238, 198, 86))
    # Flame head & crown hair
    c.oval(18, 11, 5, 5, fire_mid)
    c.poly([(13, 8), (16, 1 + abs(sway)), (18, 6), (20, 2), (23, 8)], fire_core)
    c.rect(15, 10, 2, 2, (255, 255, 245))
    c.rect(19, 10, 2, 2, (255, 255, 245))
    return c.finish((120, 28, 18, 255))


# ---------------------------------------------------------------------------
# Expressive 48x48 Pixel-Art Bust Portraits
# ---------------------------------------------------------------------------

def draw_portrait(kind, expr="normal"):
    """48x48 crisp pixel-art bust portrait for Undertale-style dialogue."""
    c = Canvas(48, 48)
    bg_map = {
        "river": (34, 48, 88), "nadia": (64, 40, 88), "grandparent": (78, 56, 46), "mark": (48, 56, 74),
        "kenny": (24, 82, 86), "guard_a": (28, 72, 92), "guard_b": (32, 54, 104),
        "queen_crab": (88, 22, 46), "sophie": (74, 34, 62),
        "jack": (18, 48, 88), "jill": (28, 42, 86), "angelfish": (24, 58, 92), "stonefish": (30, 56, 70),
        "gill": (18, 66, 86), "scal": (22, 52, 90), "lady": (34, 58, 86), "towtou": (20, 54, 66),
        "angler": (26, 18, 52), "thaumiel": (18, 32, 48), "tenebris": (18, 16, 36), "genie": (48, 18, 28),
    }
    bg = bg_map.get(kind, (36, 26, 58))
    c.rect(0, 0, 48, 48, bg)
    c.rect(2, 2, 44, 44, shade(bg, 1.18))

    # Helper to draw shoulders + neck + face
    def bust(skin, shirt, hair, eye_col=(34, 24, 52), long_hair=False, specs=False, beard=False, crown=False, floral=False):
        c.poly([(8, 47), (14, 34), (34, 34), (40, 47)], shirt)
        c.rect(16, 34, 16, 13, shade(shirt, 1.16))
        if floral:
            c.px(20, 40, (255, 198, 92)); c.px(21, 41, (255, 120, 170))
            c.px(27, 40, (140, 232, 160))
        c.rect(20, 29, 8, 7, shade(skin, 0.85))
        if long_hair:
            c.rect(10, 14, 28, 24, shade(hair, 0.78))
        c.oval(24, 21, 10, 10, skin)
        c.oval(24, 12, 11, 5, hair)
        c.rect(13, 12, 7, 4, shade(hair, 1.2))
        c.rect(13, 14, 3, 7, shade(hair, 0.8))
        c.rect(32, 14, 3, 7, hair)
        # Eyes
        if expr == "sick":
            c.rect(18, 21, 4, 1, (56, 38, 64))
            c.rect(26, 21, 4, 1, (56, 38, 64))
            c.rect(16, 24, 3, 2, (236, 150, 170))
            c.rect(29, 24, 3, 2, (236, 150, 170))
        else:
            for ex in (18, 26):
                c.rect(ex, 19, 4, 4, (250, 252, 255))
                c.rect(ex + 1, 20, 2, 3, eye_col)
                c.px(ex + 1, 20, (255, 255, 255))
            if expr == "shocked":
                c.rect(18, 16, 4, 1, (48, 28, 52))
                c.rect(26, 16, 4, 1, (48, 28, 52))
        if specs:
            c.rect(17, 18, 6, 6, (220, 240, 255, 110))
            c.rect(25, 18, 6, 6, (220, 240, 255, 110))
            c.rect(23, 20, 2, 1, (196, 160, 82))
        if beard:
            c.oval(24, 29, 8, 5, hair)
        # Mouth
        if expr == "cured" or expr == "happy":
            c.line(21, 27, 24, 29, (110, 44, 58))
            c.line(24, 29, 27, 27, (110, 44, 58))
        elif expr == "shocked":
            c.oval(24, 28, 2, 2, (110, 44, 58))
        elif expr == "sick":
            c.line(22, 28, 26, 28, (130, 68, 86))
        else:
            c.line(21, 27, 27, 27, (110, 48, 58))
        if crown:
            c.rect(17, 6, 14, 4, (244, 198, 72))

    if kind == "river":
        bust((242, 196, 160), (46, 134, 232), (107, 68, 46))
        c.rect(20, 34, 8, 2, (236, 242, 250))  # collar
    elif kind == "nadia":
        bust((246, 204, 174), (118, 58, 168), (242, 206, 98), long_hair=True, floral=True)
    elif kind == "grandparent":
        bust((232, 184, 152), (156, 112, 74), (192, 198, 206), specs=True, beard=True)
    elif kind == "mark":
        bust((232, 184, 150), (74, 92, 116), (62, 42, 34))
    elif kind == "kenny":
        shell = (224, 84, 42)
        c.oval(24, 30, 14, 12, shell)
        c.oval(22, 25, 9, 6, (255, 138, 76))
        c.line(12, 22, 36, 40, (74, 168, 92), width=4)  # kelp slingbag strap!
        c.rect(16, 11, 3, 9, shell); c.rect(29, 11, 3, 9, shell)
        for ex in (15, 28):
            c.oval(ex + 1, 10, 3, 3, (252, 252, 255))
            c.rect(ex + 1, 10, 2, 2, (32, 20, 40))
        c.oval(9, 24, 5, 5, (255, 138, 76))
        c.oval(39, 24, 5, 5, (255, 138, 76))
        if expr == "happy":
            c.line(19, 31, 24, 34, (64, 20, 14)); c.line(24, 34, 29, 31, (64, 20, 14))
        elif expr == "worried":
            c.line(19, 33, 29, 31, (64, 20, 14))
        else:
            c.line(19, 32, 29, 32, (64, 20, 14))
    elif kind in ("guard_a", "guard_b"):
        col = (76, 194, 168) if kind == "guard_a" else (92, 126, 232)
        c.poly([(12, 47), (16, 30), (32, 30), (36, 47)], col)
        c.oval(24, 22, 9, 8, col)
        c.rect(31, 21, 9, 4, shade(col, 1.15))
        c.rect(15, 10, 18, 7, (218, 178, 86))
        c.poly([(16, 10), (24, 4), (32, 10)], (216, 52, 88))
        c.rect(22, 19, 4, 4, (250, 252, 255)); c.rect(23, 20, 2, 2, (24, 24, 48))
    elif kind == "queen_crab":
        shell = (242, 62, 52) if expr == "fuming" else (214, 65, 40)
        c.poly([(8, 47), (13, 31), (35, 31), (40, 47)], (118, 22, 56))
        c.oval(24, 25, 14, 11, shell)
        for sx, sy in ((11, 18), (15, 13), (33, 13), (37, 18)):
            c.poly([(sx - 2, sy + 2), (sx, sy - 4), (sx + 2, sy + 2)], (146, 30, 22))
        c.rect(16, 7, 16, 5, (244, 198, 72))
        for cx in (16, 23, 30):
            c.rect(cx, 3, 2, 4, (244, 198, 72))
        c.px(24, 9, (255, 60, 140))
        # Glowing pearl necklace
        for nx in range(16, 33, 3):
            c.rect(nx, 36, 2, 2, (248, 242, 215))
        c.oval(24, 38, 3, 3, (110, 244, 255))
        c.eyes(18, 27, 21, iris=(40, 12, 20), mood="angry" if expr == "fuming" else "normal")
        if expr == "fuming":
            c.oval(9, 10, 3, 3, (255, 226, 234))
            c.oval(39, 10, 3, 3, (255, 226, 234))
            c.rect(20, 29, 8, 3, (64, 12, 18))
        else:
            c.line(20, 29, 28, 29, (64, 12, 18))
    elif kind == "sophie":
        c.poly([(10, 47), (15, 34), (33, 34), (38, 47)], (255, 122, 42))
        c.rect(20, 34, 8, 13, (250, 246, 240))
        c.oval(24, 22, 10, 9, (255, 122, 42))
        c.rect(28, 15, 3, 14, (250, 246, 240))
        c.poly([(15, 15), (9, 5), (22, 12)], (216, 48, 86))
        c.poly([(33, 15), (39, 5), (26, 12)], (82, 194, 130))
        c.rect(8, 5, 2, 2, (250, 216, 86)); c.rect(38, 5, 2, 2, (250, 216, 86))
        c.eyes(18, 26, 20, iris=(28, 22, 44))
    elif kind in ("jack", "jill"):
        col = (64, 126, 198) if kind == "jack" else (88, 146, 216)
        suit = (34, 46, 76) if kind == "jack" else (112, 46, 84)
        c.poly([(9, 47), (15, 33), (33, 33), (39, 47)], suit)
        c.rect(20, 33, 8, 14, (242, 246, 252))
        c.rect(23, 34, 2, 12, (200, 62, 82) if kind == "jack" else (240, 198, 92))
        # Tall sail crest + billfish face + rostrum
        crest_y = 1 if expr == "shocked" else 3
        c.poly([(15, 15), (23, crest_y), (28, 15)], (42, 84, 156))
        c.oval(23, 23, 9, 9, col)
        c.poly([(30, 22), (45, 25), (30, 27)], (188, 218, 244))
        if expr == "shocked":
            c.rect(16, 17, 5, 6, (252, 253, 255))
            c.rect(24, 17, 5, 6, (252, 253, 255))
            c.rect(18, 19, 2, 2, (20, 28, 54))
            c.rect(26, 19, 2, 2, (20, 28, 54))
            c.oval(24, 29, 2, 2, (24, 22, 46))
            c.rect(8, 10, 2, 6, (255, 224, 102))
            c.rect(8, 18, 2, 2, (255, 224, 102))
        else:
            c.eyes(18, 25, 20, iris=(20, 28, 54), mood="normal")
            c.line(21, 29, 27, 29, (24, 28, 52))
        if kind == "jack":
            c.rect(24, 19, 5, 5, (238, 202, 102))
            c.rect(25, 20, 3, 3, (176, 242, 255))
    elif kind == "towtou":
        c.poly([(8, 47), (14, 34), (34, 34), (40, 47)], (42, 74, 80))
        c.oval(24, 23, 10, 9, (72, 118, 108))
        c.oval(24, 31, 9, 6, (238, 240, 245))  # beard
        c.oval(23, 14, 11, 4, (196, 44, 72))  # beret
        c.eyes(18, 26, 21, iris=(24, 24, 40))
        c.line(17, 17, 22, 25, (198, 62, 76), width=2)  # scar
    elif kind == "thaumiel":
        c.poly([(8, 47), (15, 32), (33, 32), (40, 47)], (64, 78, 62))
        c.oval(24, 23, 10, 9, (92, 116, 134))
        c.line(23, 14, 30, 7, (56, 70, 96), width=2)
        c.line(30, 7, 36, 15, (56, 70, 96), width=2)
        c.oval(36, 17, 4, 4, (104, 244, 255))
        c.oval(36, 17, 2, 2, (255, 255, 255))
        c.eyes(18, 26, 21, iris=(110, 244, 255))
    elif kind == "tenebris":
        c.poly([(10, 47), (16, 35), (32, 35), (38, 47)], (28, 26, 42))
        c.oval(24, 26, 8, 7, (62, 68, 96))
        c.rect(11, 18, 26, 3, (24, 22, 36))  # trenchcoat hat brim
        c.rect(16, 11, 16, 7, (34, 30, 50))
        c.rect(18, 24, 4, 2, (110, 240, 255))
        c.rect(26, 24, 4, 2, (110, 240, 255))
    elif kind == "genie":
        c.poly([(10, 47), (16, 31), (32, 31), (38, 47)], (242, 92, 34))
        c.oval(24, 22, 10, 9, (255, 168, 54))
        c.poly([(14, 16), (19, 5), (24, 13), (29, 4), (34, 16)], (255, 240, 142))
        c.rect(18, 20, 4, 3, (255, 255, 245))
        c.rect(26, 20, 4, 3, (255, 255, 245))
    elif kind in ("angelfish", "stonefish", "gill", "scal", "lady", "angler"):
        col_map = {
            "angelfish": ((230, 240, 250), (42, 54, 92)),
            "stonefish": ((124, 106, 76), (242, 244, 248)),
            "gill": ((62, 184, 168), (38, 94, 118)),
            "scal": ((74, 118, 176), (56, 76, 112)),
            "lady": ((236, 132, 146), (186, 72, 106)),
            "angler": ((82, 64, 136), (48, 30, 82)),
        }
        sk, sh = col_map[kind]
        c.poly([(10, 47), (15, 34), (33, 34), (38, 47)], sh)
        if kind in ("gill", "scal", "lady"):
            fc = (38, 134, 124) if kind == "gill" else ((96, 198, 190) if kind == "scal" else (196, 92, 110))
            c.poly([(14, 18), (7, 23), (14, 28)], fc)
            c.poly([(34, 18), (41, 23), (34, 28)], fc)
        c.oval(24, 23, 10, 9, sk)
        if kind == "angelfish":
            c.poly([(15, 15), (24, 3), (33, 15)], (245, 248, 255))
            c.rect(12, 16, 4, 12, (240, 244, 252))
            c.rect(32, 16, 4, 12, (220, 228, 242))
            c.rect(18, 34, 12, 6, (255, 255, 255))
        elif kind == "stonefish":
            c.rect(15, 12, 18, 5, (190, 62, 74))  # backwards red cap
            c.rect(10, 14, 7, 3, (150, 44, 56))
            c.line(19, 27, 29, 27, (68, 54, 36))
        elif kind == "scal":
            c.oval(24, 15, 9, 4, (124, 152, 186))
            c.line(20, 28, 28, 28, (32, 54, 86))
        elif kind == "gill":
            c.poly([(19, 14), (24, 7), (29, 14)], (38, 134, 124))
            c.line(19, 27, 24, 29, (22, 68, 72))
            c.line(24, 29, 29, 27, (22, 68, 72))
        elif kind == "lady":
            c.rect(12, 33, 24, 7, (244, 190, 162))  # cozy market shawl
            c.oval(14, 16, 8, 4, (238, 242, 248))  # silver curls
            c.oval(34, 16, 4, 4, (238, 242, 248))
            c.line(20, 27, 24, 29, (124, 42, 62))
            c.line(24, 29, 28, 27, (124, 42, 62))
        elif kind == "angler":
            c.line(24, 14, 33, 9, (56, 70, 96), width=2)
            c.oval(35, 13, 3, 3, (255, 110, 210))
            c.line(20, 28, 28, 27, (42, 22, 62))
        c.eyes(18, 26, 21, iris=(24, 24, 48))

    return outline_image(c.im, OUTLINE_ABYSS) if kind in ("thaumiel", "tenebris", "angler") else c.im


# ---------------------------------------------------------------------------
# Intro Storybook Plates (240x120 -> displayed at 3x = 720x360)
# ---------------------------------------------------------------------------

def draw_intro_plates():
    plates = []
    # Plate 0: Nadia sick in bed, blurred unreadable letters drifting off her book
    c = Canvas(240, 120)
    c.rect(0, 0, 240, 120, (44, 28, 56))
    c.rect(0, 86, 240, 34, (74, 48, 62))
    # Window with crescent moon
    c.rect(24, 16, 44, 46, (22, 24, 54))
    c.rect(22, 14, 48, 4, (118, 78, 56)); c.rect(22, 62, 48, 4, (118, 78, 56))
    c.rect(45, 16, 2, 46, (118, 78, 56)); c.rect(24, 38, 44, 2, (118, 78, 56))
    c.oval(36, 26, 5, 5, (250, 232, 162)); c.oval(38, 25, 4, 4, (22, 24, 54))
    # Bed + Nadia
    c.rect(94, 58, 92, 34, (114, 68, 54))
    c.rect(90, 48, 8, 48, (92, 54, 42))
    c.rect(100, 64, 84, 22, (148, 92, 186))
    c.rect(98, 58, 22, 10, (246, 240, 230))
    c.oval(114, 58, 9, 8, (246, 204, 174))
    c.oval(114, 52, 10, 5, (242, 206, 98))
    c.rect(110, 58, 4, 1, (54, 34, 62)); c.rect(116, 58, 4, 1, (54, 34, 62))
    # Open book with drifting smudged glyphs
    c.rect(144, 56, 24, 14, (246, 236, 214))
    c.rect(155, 56, 2, 14, (176, 152, 128))
    for gx, gy in ((148, 44), (158, 38), (166, 46), (152, 30), (162, 26)):
        c.rect(gx, gy, 5, 2, (236, 152, 206, 180))
    plates.append(c.im)

    # Plate 1: Mark & River working hard / studying late by warm lamp
    c = Canvas(240, 120)
    c.rect(0, 0, 240, 120, (36, 30, 52))
    c.rect(0, 90, 240, 30, (82, 54, 44))
    # Warm lamp pool
    c.oval(160, 52, 56, 44, (124, 86, 58))
    c.oval(160, 52, 34, 28, (186, 134, 72))
    # Desk & Lamp & Books
    c.rect(120, 68, 92, 24, (118, 78, 52))
    c.rect(182, 48, 12, 10, (250, 214, 122))
    c.rect(187, 58, 2, 10, (90, 74, 86))
    c.rect(134, 62, 20, 6, (46, 134, 232))
    # River seated tired at desk
    c.rect(124, 50, 16, 18, (46, 134, 232))
    c.oval(134, 42, 9, 8, (242, 196, 160))
    c.oval(134, 36, 10, 5, (107, 68, 46))
    # Mark in background doorway with tool crate
    c.rect(42, 30, 38, 60, (54, 40, 62))
    c.rect(52, 48, 16, 22, (74, 92, 116))
    c.oval(60, 40, 8, 7, (232, 184, 150))
    c.oval(60, 35, 9, 4, (62, 42, 34))
    c.rect(62, 56, 14, 10, (148, 102, 64))
    plates.append(c.im)

    # Plate 2: Moonlit beach, River resting exhausted as glowing book washes ashore
    c = Canvas(240, 120)
    c.rect(0, 0, 240, 62, (16, 24, 58))
    c.oval(188, 22, 12, 12, (250, 240, 196))
    for sx, sy in ((28, 16), (68, 26), (112, 12), (144, 30), (214, 14)):
        c.rect(sx, sy, 2, 2, (240, 246, 255))
    # Ocean waves & glowing shoreline
    c.rect(0, 56, 240, 24, (22, 68, 124))
    c.rect(0, 72, 240, 8, (110, 196, 224))
    c.rect(0, 80, 240, 40, (214, 186, 126))
    # River sitting on sand
    c.rect(56, 70, 14, 16, (46, 134, 232))
    c.rect(56, 84, 20, 6, (182, 137, 82))
    c.oval(63, 62, 8, 8, (242, 196, 160))
    c.oval(63, 57, 9, 4, (107, 68, 46))
    # Glowing Family Heirloom washing up in foam
    c.oval(146, 80, 18, 10, (110, 244, 255, 120))
    c.rect(136, 74, 20, 14, (96, 46, 136))
    c.rect(140, 76, 12, 10, (246, 208, 104))
    plates.append(c.im)

    # Plate 3: River asleep beside the opened glowing book, vortex pulling into the sea
    c = Canvas(240, 120)
    c.rect(0, 0, 240, 120, (12, 20, 48))
    c.oval(120, 60, 86, 50, (24, 72, 124))
    c.oval(120, 60, 58, 34, (42, 134, 178))
    c.oval(120, 60, 32, 18, (122, 236, 246))
    # Open book radiating light
    c.poly([(92, 68), (120, 72), (120, 52), (96, 50)], (252, 242, 214))
    c.poly([(148, 68), (120, 72), (120, 52), (144, 50)], (252, 242, 214))
    # River silhouette drifting into swirling runes
    c.rect(112, 24, 14, 18, (46, 134, 232))
    c.oval(119, 18, 7, 7, (242, 196, 160))
    for a in range(12):
        ang = a * math.pi / 6.0
        px = 120 + math.cos(ang) * (36 + (a % 3) * 14)
        py = 56 + math.sin(ang) * (20 + (a % 2) * 8)
        c.rect(px, py, 3, 3, (255, 224, 126) if a % 2 == 0 else (140, 248, 255))
    plates.append(c.im)
    return plates


# ---------------------------------------------------------------------------
# Biome Environment Props
# ---------------------------------------------------------------------------

def draw_props(out_dir):
    os.makedirs(out_dir, exist_ok=True)

    # 1. Giant Kelp Stalks (3 variants, 32x128)
    for vi, hue in enumerate(((52, 168, 92), (42, 144, 110), (74, 186, 84))):
        c = Canvas(32, 128)
        hi = shade(hue, 1.24); lo = shade(hue, 0.68)
        for y in range(6, 128):
            ox = int(math.sin(y * 0.11 + vi) * 3.0)
            c.rect(15 + ox, y, 3, 1, lo)
            c.px(15 + ox, y, hue)
            if y % 12 == 0 and y > 10:
                side = -1 if ((y // 12 + vi) % 2 == 0) else 1
                c.oval(16 + ox + side * 6, y, 5, 3, hue)
                c.oval(16 + ox + side * 6, y - 1, 3, 1, hi)
        c.finish(OUTLINE_SEA).save(os.path.join(out_dir, f"kelp_{vi}.png"))

    # 2. Kenny's Cozy Kelp Hut (96x80)
    c = Canvas(96, 80)
    c.oval(48, 40, 38, 26, (214, 108, 74))  # coral-shell dome
    c.oval(46, 32, 28, 16, (242, 142, 98))
    c.rect(12, 46, 72, 32, (164, 118, 78))
    c.rect(14, 48, 68, 28, (194, 146, 98))
    c.rect(38, 52, 20, 26, (92, 56, 40))  # arched wooden door
    c.oval(48, 52, 10, 8, (92, 56, 40))
    c.oval(24, 58, 7, 7, (255, 224, 130))  # warm porthole
    c.oval(72, 58, 7, 7, (255, 224, 130))
    # kelp shingle trim
    for kx in range(14, 82, 8):
        c.oval(kx, 46, 5, 4, (62, 158, 86))
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "kenny_house.png"))

    # 3. Riddle Stone Monolith (36x52)
    c = Canvas(36, 52)
    c.poly([(6, 50), (8, 12), (18, 4), (28, 12), (30, 50)], (92, 108, 132))
    c.poly([(8, 48), (10, 13), (18, 7), (25, 13), (26, 48)], (126, 144, 170))
    c.rect(12, 16, 12, 24, (58, 70, 94))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "riddle_stone.png"))

    # 4. Throne Gate Archway (128x112)
    c = Canvas(128, 112)
    c.rect(14, 18, 100, 92, (74, 86, 112))
    c.rect(18, 22, 92, 86, (104, 118, 148))
    c.poly([(10, 22), (64, 4), (118, 22)], (128, 142, 174))
    # Inner carved portal & glowing rune lintel
    c.rect(38, 42, 52, 68, (22, 26, 44))
    c.oval(64, 44, 26, 20, (22, 26, 44))
    c.rect(28, 26, 72, 6, (118, 232, 226))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "throne_gate.png"))

    # 5. Crab Castle Silhouette Facade (180x140)
    c = Canvas(180, 140)
    wall = (146, 54, 72); wall_hi = (188, 78, 96); roof = (214, 62, 52)
    c.rect(20, 48, 140, 90, wall)
    c.rect(24, 52, 132, 84, wall_hi)
    # flanking towers + tall spiky center keep
    for tx in (16, 134):
        c.rect(tx, 26, 30, 112, wall)
        c.poly([(tx - 4, 26), (tx + 15, 6), (tx + 34, 26)], roof)
    c.rect(64, 18, 52, 120, wall)
    c.poly([(60, 18), (90, 2), (120, 18)], roof)
    # claw-shaped gate arch
    c.rect(72, 86, 36, 52, (34, 18, 32))
    c.oval(90, 88, 18, 16, (34, 18, 32))
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "crab_castle.png"))

    # 6. Shipwreck Hull & Anchor (96x64)
    c = Canvas(96, 64)
    wood = (118, 78, 50); wood_lo = (78, 50, 34)
    c.poly([(6, 46), (18, 62), (76, 62), (88, 36), (62, 44)], wood)
    c.poly([(14, 48), (22, 59), (72, 59), (80, 42)], wood_lo)
    c.rect(42, 8, 5, 50, wood)  # broken mast
    c.line(24, 18, 64, 26, wood_lo, width=3)
    c.poly([(24, 20), (54, 26), (48, 40), (22, 34)], (210, 214, 198, 180))  # tattered sail
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "shipwreck.png"))

    # 7. Coral Houses & Market Stall (80x72)
    for idx, col in enumerate(((236, 118, 148), (92, 192, 206))):
        c = Canvas(80, 72)
        c.oval(40, 32, 30, 22, col)
        c.oval(36, 24, 20, 12, shade(col, 1.18))
        c.rect(14, 34, 52, 36, shade(col, 0.85))
        c.rect(32, 46, 16, 24, (44, 32, 62))
        c.oval(24, 46, 5, 5, (255, 228, 142))
        c.oval(56, 46, 5, 5, (255, 228, 142))
        c.finish(OUTLINE_SEA).save(os.path.join(out_dir, f"coral_house_{idx}.png"))

    c = Canvas(84, 68)
    c.rect(10, 26, 6, 40, (146, 98, 62)); c.rect(68, 26, 6, 40, (146, 98, 62))
    c.rect(8, 44, 68, 22, (168, 118, 72))
    for i in range(6):
        col = (242, 92, 134) if i % 2 == 0 else (250, 242, 224)
        c.rect(6 + i * 12, 14, 12, 14, col)
    # Market wares: blue seashroom, tentacle anemone, sea berries
    c.oval(24, 42, 5, 4, (78, 178, 248)); c.oval(42, 42, 5, 4, (218, 96, 188))
    c.oval(60, 42, 5, 4, (146, 228, 140))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "market_stall.png"))

    # 8. Tow-Tou's Scrapyard Garage (120x84)
    c = Canvas(120, 84)
    c.rect(14, 16, 92, 66, (76, 86, 98))
    c.rect(18, 20, 84, 58, (98, 112, 126))
    for sx in range(16, 104, 8):
        c.line(sx, 16, sx, 82, (62, 72, 82))
    c.rect(32, 34, 56, 48, (30, 36, 46))  # open garage bay
    for gy in range(36, 50, 4):
        c.rect(34, gy, 52, 2, (124, 136, 148))
    c.rect(42, 22, 36, 10, (214, 146, 68))  # rusted sign
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "towtou_garage.png"))

    # 9. Whale Skeleton & Somber Slum Shack & Algae Temple
    c = Canvas(144, 68)
    bone = (212, 218, 226); bone_lo = (146, 156, 176)
    c.oval(28, 42, 22, 16, bone)
    c.oval(24, 38, 14, 10, shade(bone, 1.12))
    c.oval(20, 40, 4, 4, (18, 20, 36))  # eye socket
    c.rect(44, 34, 88, 5, bone_lo)
    for rx in range(52, 126, 10):
        c.poly([(rx, 36), (rx + 4, 36), (rx + 8, 62), (rx + 2, 62)], bone)
    c.finish(OUTLINE_ABYSS).save(os.path.join(out_dir, "whale_skeleton.png"))

    c = Canvas(76, 68)
    c.rect(10, 22, 56, 44, (46, 42, 64))
    c.poly([(4, 24), (38, 6), (72, 24)], (62, 56, 84))
    c.rect(28, 38, 16, 28, (26, 22, 38))
    c.rect(48, 34, 12, 12, (110, 232, 246))  # bioluminescent window
    c.finish(OUTLINE_ABYSS).save(os.path.join(out_dir, "slum_shack.png"))

    c = Canvas(112, 96)
    st = (58, 56, 92); st_hi = (88, 86, 134)
    c.poly([(8, 28), (56, 6), (104, 28)], st_hi)
    c.rect(14, 28, 84, 66, st)
    for px in (18, 40, 64, 86):
        c.rect(px, 28, 8, 66, st_hi)
    c.rect(42, 48, 28, 46, (16, 14, 30))
    c.oval(56, 70, 8, 10, (82, 246, 206))  # Figurate Algae shrine glow!
    c.finish(OUTLINE_ABYSS).save(os.path.join(out_dir, "algae_temple.png"))

    # 10. Torch Podiums + Stone Circle Portal + Home Furniture
    for lit in (0, 1, 2):
        c = Canvas(32, 56)
        c.rect(8, 38, 16, 16, (64, 58, 96))
        c.rect(13, 18, 6, 20, (88, 82, 124))
        c.rect(9, 14, 14, 6, (112, 104, 152))
        if lit > 0:
            dy = 0 if lit == 1 else -1
            c.poly([(16, 2 + dy), (9, 14), (23, 14)], (255, 118, 46))
            c.poly([(16, 5 + dy), (12, 14), (20, 14)], (255, 226, 116))
        c.finish(OUTLINE_ABYSS).save(os.path.join(out_dir, f"torch_{lit}.png"))

    c = Canvas(112, 112)
    c.oval(56, 56, 46, 46, (44, 38, 76))
    c.oval(56, 56, 34, 34, (12, 10, 24))
    for i in range(8):
        ang = i * math.pi / 4.0
        rx, ry = 56 + math.cos(ang) * 40, 56 + math.sin(ang) * 40
        c.rect(rx - 5, ry - 5, 10, 10, (112, 98, 164))
        c.px(rx, ry, (110, 244, 255))
    c.finish(OUTLINE_ABYSS).save(os.path.join(out_dir, "portal_ring.png"))

    # Home Bed (72x44), Hearth (48x64), Bookshelf (44x56), Table (52x32)
    c = Canvas(72, 44)
    c.rect(4, 10, 8, 32, (118, 74, 48))
    c.rect(60, 18, 8, 24, (118, 74, 48))
    c.rect(10, 22, 52, 18, (148, 96, 62))
    c.rect(12, 18, 14, 8, (246, 240, 230))  # pillow
    c.rect(24, 18, 36, 14, (148, 86, 194))  # purple quilt
    c.rect(24, 18, 36, 4, (194, 134, 232))
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "home_bed.png"))

    c = Canvas(48, 64)
    c.rect(6, 4, 36, 58, (116, 104, 118))
    c.rect(4, 24, 40, 5, (148, 100, 66))
    c.rect(14, 34, 20, 28, (32, 22, 30))
    c.poly([(24, 38), (16, 58), (32, 58)], (255, 134, 48))
    c.poly([(24, 44), (19, 58), (29, 58)], (255, 224, 118))
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "home_hearth.png"))

    c = Canvas(44, 56)
    c.rect(4, 4, 36, 50, (124, 82, 52))
    c.rect(7, 8, 30, 42, (74, 46, 30))
    for sy in (10, 24, 38):
        c.rect(7, sy + 10, 30, 3, (148, 98, 64))
        for bx in range(9, 34, 5):
            col = [(220, 86, 116), (86, 174, 224), (242, 198, 86), (116, 202, 132)][(bx + sy) % 4]
            c.rect(bx, sy + 1, 4, 9, col)
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "home_shelf.png"))

    c = Canvas(52, 36)
    c.rect(4, 12, 44, 6, (154, 104, 68))
    c.rect(8, 18, 5, 16, (116, 74, 48))
    c.rect(39, 18, 5, 16, (116, 74, 48))
    # Heirloom book resting on table
    c.rect(18, 4, 16, 8, (108, 54, 152))
    c.rect(20, 5, 12, 4, (244, 198, 82))
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "home_table.png"))


def generate_all(char_dir, portrait_dir, intro_dir, props_dir):
    os.makedirs(char_dir, exist_ok=True)
    os.makedirs(portrait_dir, exist_ok=True)
    os.makedirs(intro_dir, exist_ok=True)
    os.makedirs(props_dir, exist_ok=True)

    humanoid_specs = {
        "river": {"skin": (242, 196, 160), "shirt": (46, 134, 232), "pants": (182, 137, 82), "hair": (107, 68, 46), "collar": (236, 242, 250)},
        "nadia": {"skin": (246, 204, 174), "shirt": (118, 58, 168), "pants": (78, 110, 158), "hair": (242, 206, 98), "short": 4, "long_hair": True, "floral": True},
        "grandparent": {"skin": (232, 184, 152), "shirt": (156, 112, 74), "pants": (104, 106, 118), "hair": (192, 198, 206), "glasses": True, "beard": True},
        "mark": {"skin": (232, 184, 150), "shirt": (74, 92, 116), "pants": (62, 66, 84), "hair": (62, 42, 34)},
        "jack": {"species": "billfish", "skin": (64, 126, 198), "fin": (42, 84, 156), "shirt": (34, 46, 76), "pants": (28, 36, 56), "hair": (42, 84, 156), "collar": (242, 246, 252), "tie": (200, 62, 82), "monocle": True},
        "jill": {"species": "billfish", "skin": (88, 146, 216), "fin": (56, 108, 178), "shirt": (112, 46, 84), "pants": (42, 32, 54), "hair": (56, 108, 178), "collar": (246, 232, 240), "tie": (240, 198, 92)},
        "angelfish": {"species": "angelfish", "skin": (230, 240, 250), "shirt": (42, 54, 92), "pants": (238, 242, 248), "hair": (245, 248, 255)},
        "stonefish": {"species": "stonefish", "skin": (124, 106, 76), "shirt": (242, 244, 248), "pants": (46, 58, 92), "hair": (190, 62, 74)},
        "gill": {"species": "reef", "skin": (62, 184, 168), "fin": (38, 134, 124), "shirt": (38, 94, 118), "pants": (48, 72, 92), "hair": (38, 134, 124)},
        "scal": {"species": "reef", "skin": (74, 118, 176), "fin": (96, 198, 190), "shirt": (56, 76, 112), "pants": (42, 56, 86), "hair": (56, 76, 112), "helmet": True},
        "lady": {"species": "reef", "skin": (236, 132, 146), "fin": (196, 92, 110), "shirt": (186, 72, 106), "pants": (96, 54, 78), "hair": (186, 72, 106), "shawl": (244, 190, 162)},
        "angler_a": {"species": "angler", "skin": (82, 64, 136), "lure": (104, 244, 255), "shirt": (214, 62, 146), "pants": (42, 34, 68), "hair": (42, 34, 68), "hood": (162, 42, 110), "outline": OUTLINE_ABYSS},
        "angler_b": {"species": "angler", "skin": (74, 58, 124), "lure": (255, 110, 210), "shirt": (54, 142, 194), "pants": (34, 34, 56), "hair": (34, 34, 56), "hood": (38, 98, 146), "outline": OUTLINE_ABYSS},
        "angler_c": {"species": "angler", "skin": (96, 72, 148), "lure": (255, 224, 102), "shirt": (118, 72, 186), "pants": (36, 28, 56), "hair": (36, 28, 56), "hood": (82, 46, 138), "outline": OUTLINE_ABYSS},
        "thaumiel": {"species": "angler", "skin": (92, 116, 134), "lure": (104, 244, 255), "shirt": (64, 78, 62), "pants": (44, 52, 44), "hair": (64, 78, 62), "hood": (76, 92, 70), "outline": OUTLINE_ABYSS},
        "tenebris": {"species": "angler", "skin": (62, 68, 96), "lure": (255, 96, 136), "shirt": (28, 26, 42), "pants": (22, 20, 34), "hair": (24, 22, 36), "short": 8, "trenchcoat": True, "hat": True, "sunglasses": True, "outline": OUTLINE_ABYSS},
    }

    for cid, cfg in humanoid_specs.items():
        for pose in ("stand", "w1", "w2"):
            im = draw_humanoid(cfg, pose)
            fn = cid if pose == "stand" else f"{cid}_{pose}"
            im.save(os.path.join(char_dir, f"{fn}.png"))
        make_shadow(draw_humanoid(cfg, "stand")).save(os.path.join(char_dir, f"{cid}_shadow.png"))

    # River swimming frames
    draw_river_swim("swim1").save(os.path.join(char_dir, "river_swim1.png"))
    draw_river_swim("swim2").save(os.path.join(char_dir, "river_swim2.png"))

    # Bespoke non-humanoid cast
    custom_fns = {
        "kenny": draw_kenny,
        "guard_a": lambda p: draw_seahorse_guard(0, p),
        "guard_b": lambda p: draw_seahorse_guard(1, p),
        "queen_crab": draw_queen_crab,
        "sophie": draw_sophie,
        "towtou": draw_towtou,
        "genie": draw_genie,
    }
    for cid, fn in custom_fns.items():
        for pose in ("stand", "w1", "w2"):
            im = fn(pose)
            out_name = cid if pose == "stand" else f"{cid}_{pose}"
            im.save(os.path.join(char_dir, f"{out_name}.png"))
        make_shadow(fn("stand")).save(os.path.join(char_dir, f"{cid}_shadow.png"))

    # Expressive portraits
    portrait_specs = [
        ("river", ["normal", "shocked"]),
        ("nadia", ["sick", "cured"]),
        ("grandparent", ["normal"]),
        ("mark", ["normal"]),
        ("kenny", ["normal", "happy", "worried"]),
        ("guard_a", ["normal"]),
        ("guard_b", ["normal"]),
        ("queen_crab", ["normal", "fuming"]),
        ("sophie", ["normal"]),
        ("jack", ["normal"]),
        ("jill", ["normal", "shocked"]),
        ("angelfish", ["normal"]),
        ("stonefish", ["normal"]),
        ("gill", ["normal"]),
        ("scal", ["normal"]),
        ("lady", ["normal"]),
        ("towtou", ["normal"]),
        ("angler", ["normal"]),
        ("thaumiel", ["normal"]),
        ("tenebris", ["normal"]),
        ("genie", ["normal"]),
    ]
    for kind, exprs in portrait_specs:
        for ex in exprs:
            draw_portrait(kind, ex).save(os.path.join(portrait_dir, f"{kind}_{ex}.png"))

    # Intro plates
    for idx, plate in enumerate(draw_intro_plates()):
        plate.save(os.path.join(intro_dir, f"plate_{idx}.png"))

    # Biome props
    draw_props(props_dir)
    print(f"pixel_cast: {len(humanoid_specs) + len(custom_fns)} characters, portraits, 4 intro plates, props emitted")
