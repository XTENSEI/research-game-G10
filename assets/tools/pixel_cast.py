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
    """Hue-shifted 3-stop shading: highlights warm toward cream/yellow, shadows cool toward violet."""
    r, g, b = col[:3]
    a = col[3] if len(col) > 3 else 255
    if factor >= 1.0:
        t = min(0.45, (factor - 1.0) * 1.1)
        nr = int(min(255, r * factor * (1.0 - t) + 255 * t))
        ng = int(min(255, g * factor * (1.0 - t) + 242 * t))
        nb = int(min(255, b * factor * (1.0 - t) + 196 * t))
        return (nr, ng, nb, a)
    t = min(0.45, (1.0 - factor) * 1.1)
    nr = int(max(0, r * factor * (1.0 - t) + 24 * t))
    ng = int(max(0, g * factor * (1.0 - t) + 18 * t))
    nb = int(max(0, b * factor * (1.0 - t) + 52 * t))
    return (nr, ng, nb, a)


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
    """Draw 32x38 humanoid in crisp 3/4 right-facing profile with true 4-phase gait and zero waist/joint gaps."""
    W, H = 32, 38
    c = Canvas(W, H)
    # 4-phase stride:
    # - w1: near leg forward contact, far leg pushing back (bob=0)
    # - w2: passing midstride, far knee lifted forward (bob=-1)
    # - w3: far leg forward contact, near leg pushing back (bob=0)
    # - w4: opposite passing recoil, near knee lifted (bob=+1)
    # - idle: relaxed exhale + eye blink (bob=+1)
    bob = {"stand": 0, "idle": 1, "w1": 0, "w2": -1, "w3": 0, "w4": 1}.get(pose, 0)
    short = cfg.get("short", 0)
    skin = cfg["skin"]
    skin_hi = shade(skin, 1.09)
    skin_lo = shade(skin, 0.80)
    shirt = cfg["shirt"]
    shirt_hi = shade(shirt, 1.24)
    shirt_lo = shade(shirt, 0.72)
    pants = cfg["pants"]
    pants_hi = shade(pants, 1.15)
    pants_lo = shade(pants, 0.70)
    hair = cfg["hair"]
    hair_hi = shade(hair, 1.26)
    hair_lo = shade(hair, 0.68)
    shoes = cfg.get("shoes", (72, 48, 38))
    shoes_hi = shade(shoes, 1.25)

    base_y = H - 2
    if pose == "w1":
        far_x, far_y = 8, base_y - 9
        near_x, near_y = 16, base_y - 10
        arm_off = -3
    elif pose == "w2":
        far_x, far_y = 13, base_y - 11
        near_x, near_y = 12, base_y - 9
        arm_off = -1
    elif pose == "w3":
        far_x, far_y = 16, base_y - 10
        near_x, near_y = 8, base_y - 9
        arm_off = 3
    elif pose == "w4":
        far_x, far_y = 12, base_y - 9
        near_x, near_y = 13, base_y - 11
        arm_off = 1
    else:
        far_x, far_y = 15, base_y - 9
        near_x, near_y = 11, base_y - 9
        arm_off = 0

    # Far leg (extend top up 2px to overlap pelvis seamlessly even on bob=-1)
    c.rect(far_x, far_y - 2, 4, 7, pants_lo)
    c.rect(far_x, far_y + 5, 5, 3, shade(shoes, 0.78))
    c.px(far_x + 3, far_y + 5, shoes)

    # Near leg with knee highlight and crisp heel/toe silhouette
    near_shoe_w = 5 if pose == "w3" else 6
    c.rect(near_x, near_y - 2, 4, 7, pants)
    c.rect(near_x, near_y - 1, 2, 5, pants_hi)
    c.rect(near_x, near_y + 5, near_shoe_w, 3, shoes)
    c.px(near_x + 2, near_y + 5, shoes_hi)

    # Far arm tucked on +X chest side (swings opposite near arm)
    ty = 16 + bob + short
    c.rect(19 - arm_off // 2, ty + 1 - arm_off // 2, 3, 7, shirt_lo)
    c.rect(19 - arm_off // 2, ty + 7 - arm_off // 2, 3, 2, skin_lo)

    # Pelvis bridge guaranteeing zero 1px waist tear regardless of stride or bob
    c.rect(10, ty + 9 - short // 2, 11, 3, pants_lo)

    # 3/4 right-angled torso with wrapped top-left rim & crisp waist belt
    if cfg.get("trenchcoat"):
        flap = 2 if pose in ("w1", "w3") else (1 if pose in ("w2", "w4") else 0)
        c.poly([(9, ty), (21, ty), (22 + flap, ty + 15 - short // 2), (8 - flap, ty + 15 - short // 2)], shirt)
        c.rect(9, ty, 4, 14 - short // 2, shirt_hi)
        c.rect(19, ty + 2, 2, 12 - short // 2, shirt_lo)
        c.rect(10, ty + 6, 11, 2, cfg.get("trim", (96, 198, 220)))
        c.px(17, ty + 6, (238, 206, 104))
    else:
        c.rect(10, ty, 11, 12 - short // 2, shirt)
        c.rect(10, ty, 3, 8 - short // 2, shirt_hi)
        c.rect(19, ty + 2, 2, 8 - short // 2, shirt_lo)
        c.rect(10, ty + 10 - short // 2, 11, 2, pants_lo)
        c.rect(16, ty + 10 - short // 2, 2, 2, (214, 178, 92))
        if cfg.get("collar"):
            c.rect(14, ty, 6, 2, cfg["collar"])
            c.px(16, ty + 2, cfg["collar"])
        if cfg.get("tie"):
            c.rect(16, ty + 2, 2, 6, cfg["tie"])
        if cfg.get("floral"):
            c.px(12, ty + 4, (255, 196, 98))
            c.px(14, ty + 5, (255, 120, 170))
            c.px(18, ty + 4, (136, 232, 160))
            c.rect(14, ty + 10 - short // 2, 5, 1, (220, 160, 240))

    # Near/front shoulder & swinging arm on the -X viewer side with sleeve cuff
    c.rect(8 + arm_off // 2, ty + 1 + arm_off // 2, 4, 7, shirt)
    c.rect(8 + arm_off // 2, ty + 1 + arm_off // 2, 2, 5, shirt_hi)
    c.rect(9 + arm_off // 2, ty + 7 + arm_off // 2, 3, 2, skin)

    # Smooth rounded 3/4 head silhouette
    hy = 4 + bob + short
    c.rect(10, hy + 2, 11, 10, skin)
    c.rect(11, hy + 4, 10, 7, skin)
    c.rect(12, hy + 11, 8, 2, skin_lo)
    c.rect(11, hy + 2, 6, 2, skin_hi)
    c.rect(9, hy + 6, 2, 3, skin_lo)
    c.px(9, hy + 6, skin)
    if cfg.get("species", "human") == "human":
        c.px(13, hy + 9, shade(skin, 0.92))
        c.line(16, hy + 10, 18, hy + 10, (148, 86, 78))

    eye_col = cfg.get("eye", (40, 28, 60))
    eye_mood = "closed" if pose == "idle" else "normal"
    if cfg.get("sunglasses"):
        c.rect(13, hy + 5, 4, 3, (18, 22, 40))
        c.rect(18, hy + 5, 4, 3, (18, 22, 40))
        c.rect(17, hy + 6, 1, 1, (100, 220, 255))
        c.px(14, hy + 5, (110, 240, 255))
        c.px(19, hy + 5, (110, 240, 255))
    else:
        c.eyes(14, 19, hy + 5, iris=eye_col, mood=eye_mood)

    species = cfg.get("species", "human")
    hair_follow = 1 if pose == "w2" else (-1 if pose == "w4" else 0)
    if species == "human":
        c.rect(9, hy + hair_follow, 12, 4, hair)
        c.rect(9, hy + 2 + hair_follow, 13, 3, hair)
        c.rect(10, hy - 1 + hair_follow, 5, 2, hair_hi)
        c.rect(16, hy + hair_follow, 4, 2, hair_hi)
        c.rect(14, hy + 3 + hair_follow, 3, 2, hair)
        c.rect(18, hy + 3 + hair_follow, 3, 2, hair_hi)
        c.rect(9, hy + 4, 3, 6, hair_lo)
        c.rect(11, hy + 4, 2, 4, hair)
        if cfg.get("long_hair"):
            sway_h = -1 if pose in ("w1", "w3") else 0
            c.rect(7 + sway_h, hy + 4, 4, 11, hair_lo)
            c.rect(8 + sway_h, hy + 5, 3, 9, hair)
            c.px(7 + sway_h, hy + 14, hair)
        if cfg.get("beard"):
            c.poly([(13, hy + 9), (22, hy + 9), (20, hy + 14), (15, hy + 14)], hair)
            c.rect(15, hy + 10, 5, 2, hair_hi)
        if cfg.get("glasses"):
            c.rect(13, hy + 5, 4, 2, (230, 240, 255))
            c.rect(18, hy + 5, 4, 2, (230, 240, 255))
            c.px(17, hy + 5, (196, 162, 82))
    elif species == "billfish":
        flare = 1 if pose in ("w1", "w3") else (-1 if pose == "w2" else 0)
        c.poly([(10, hy + 2), (14, hy - 4 - flare), (19, hy + 2)], cfg["fin"])
        c.poly([(12, hy + 1), (14, hy - 2 - flare), (17, hy + 1)], shade(cfg["fin"], 1.28))
        c.poly([(22, hy + 6), (31, hy + 7), (22, hy + 9)], shade(skin, 1.18))
        c.px(29, hy + 7, (235, 248, 255))
        if cfg.get("monocle"):
            c.rect(18, hy + 4, 4, 4, (235, 210, 120))
            c.rect(19, hy + 5, 2, 2, (160, 240, 255))
    elif species == "angelfish":
        flare = 1 if pose in ("w2", "w4") else 0
        c.poly([(9, hy + 2), (14, hy - 4 - flare), (20, hy + 2)], (235, 244, 255))
        c.rect(8, hy + 2, 4, 9, (244, 248, 255))
        c.rect(11, ty, 7, 4, (255, 255, 255))
    elif species == "stonefish":
        c.rect(9, hy + 1, 12, 3, (190, 62, 74))
        c.rect(8, hy, 10, 2, (226, 86, 98))
        c.rect(6, hy + 2, 4, 3, (150, 44, 56))
        c.px(15, hy + 8, (112, 92, 64))
        c.px(20, hy + 9, (112, 92, 64))
    elif species == "angler":
        lure = cfg["lure"]
        lure_dx = 1 if pose == "w1" else (-1 if pose == "w3" else 0)
        lure_dy = -1 if pose == "w2" else (1 if pose == "w4" else 0)
        if cfg.get("hat"):
            # Mount lure stem cleanly off front brim so zero hollow triangle gap forms behind lure
            c.rect(7, hy + 1, 16, 2, (22, 20, 34))
            c.rect(10, hy - 2, 10, 3, (36, 32, 54))
            c.rect(10, hy, 10, 1, (90, 44, 72))
            c.line(21, hy + 1, 25 + lure_dx, hy - 1 + lure_dy, (74, 86, 118))
            c.oval(25 + lure_dx, hy - 1 + lure_dy, 2, 2, lure)
            c.px(25 + lure_dx, hy - 2 + lure_dy, (255, 255, 255))
        else:
            if cfg.get("hood"):
                c.oval(15, hy - 1, 7, 3, cfg["hood"])
                c.rect(10, hy - 2, 6, 2, shade(cfg["hood"], 1.25))
            c.line(18, hy, 25 + lure_dx, hy - 1 + lure_dy, (56, 70, 96))
            if pose in ("w1", "w3"):
                c.oval(26 + lure_dx, hy + lure_dy, 3, 3, shade(lure, 0.8))
            c.oval(26 + lure_dx, hy + lure_dy, 2, 2, lure)
            c.px(26 + lure_dx, hy - 1 + lure_dy, (255, 255, 255))
    elif species == "reef":
        fin_fl = 1 if pose in ("w1", "w3") else (-1 if pose == "w2" else 0)
        c.poly([(9, hy + 3), (4 - fin_fl, hy + 6), (9, hy + 9)], cfg["fin"])
        if cfg.get("helmet"):
            c.oval(15, hy + 1, 6, 3, (124, 152, 186))
            c.rect(12, hy, 5, 1, (186, 212, 240))
        elif cfg.get("shawl"):
            c.rect(8, ty, 13, 5, cfg["shawl"])
            c.rect(6, ty + 4, 6, 6, (176, 126, 72))
            c.px(8, ty + 3, (220, 74, 160))
            c.px(10, ty + 3, (140, 210, 255))

    return c.finish(cfg.get("outline", OUTLINE_SEA))


def draw_river_swim(pose="swim1"):
    """River swimming horizontally in a fluid 4-frame freestyle kick matching 3/4 River proportions (38x32)."""
    c = Canvas(38, 32)
    idx = {"swim1": 0, "swim2": 1, "swim3": 2, "swim4": 3}.get(pose, 0)
    kicks = [3, 0, -3, 0][idx]
    reach = [3, -1, -2, 1][idx]
    bob = [0, -1, 0, 1][idx]

    # Connected trailing water foam attached softly behind heel on power kicks (no floating 1px dot!)
    c.rect(3, 15 + kicks // 2 + bob, 9, 4, (194, 150, 96))
    c.rect(1, 15 + kicks // 2 + bob, 3, 4, (72, 48, 38))
    c.rect(4, 20 - kicks // 2 + bob, 9, 4, (152, 114, 68))
    c.rect(2, 20 - kicks // 2 + bob, 3, 4, (58, 38, 30))

    # Blue hoodie/shirt matching CAST["river"] palette + crisp white collar trim
    c.rect(11, 13 + bob, 12, 9, (46, 134, 232))
    c.rect(12, 13 + bob, 10, 3, (88, 174, 250))
    c.rect(21, 14 + bob, 2, 6, (236, 242, 250))

    # Alternating overhead catch & pull stroke arms
    far_reach = -reach
    c.rect(15 + far_reach, 11 + bob, 8, 3, (32, 94, 172))
    c.rect(19 + reach, 18 + bob, 7, 3, (46, 134, 232))
    c.rect(26 + reach, 18 + bob, 3, 3, (242, 196, 160))

    # 3/4 Head turned right with tousled brown locks & clear eyes
    c.rect(23, 8 + bob, 10, 10, (242, 196, 160))
    c.px(33, 13 + bob, (242, 196, 160))
    c.rect(22, 6 + bob, 11, 4, (107, 68, 46))
    c.rect(19, 8 + bob + (1 if idx % 2 else 0), 4, 4, (82, 50, 32))
    c.rect(24, 6 + bob, 5, 2, (140, 92, 62))
    c.eyes(26, 30, 12 + bob, iris=(40, 28, 60))
    return c.finish(OUTLINE_SEA)


def draw_kenny(pose="stand"):
    """Kenny: 4-frame scuttling Red Kelp Crab with kelp satchel & shaded snapping pincers (38x36)."""
    c = Canvas(38, 36)
    step = {"stand": 0, "idle": 0, "w1": 2, "w2": 0, "w3": -2, "w4": 0}.get(pose, 0)
    bob = {"stand": 0, "idle": 1, "w1": 0, "w2": -1, "w3": 0, "w4": 1}.get(pose, 0)
    shell = (224, 84, 42)
    shell_hi = (255, 144, 82)
    shell_lo = (162, 48, 22)
    cleft = (102, 28, 16)
    kelp = (74, 172, 92)
    kelp_hi = (120, 214, 128)
    kelp_lo = (44, 114, 62)

    # Far-side legs rooted higher under carapace rim with tapered claw tips (clean open floor spacing)
    for i, lx in enumerate((22, 26, 30)):
        off = -step if (i % 2 == 0) else step
        if pose == "w2" and i % 2 == 1:
            off = -2
        elif pose == "w4" and i % 2 == 0:
            off = -2
        c.rect(lx, 20 + off // 2, 2, 10, shell_lo)
        c.rect(lx + 1, 30 + off // 2, 1, 3, shell)

    # Angled 3/4 carapace + underside belly rim
    cy = 18 + bob
    c.oval(18, cy, 11, 8, shell)
    c.oval(15, cy - 3, 7, 4, shell_hi)
    c.px(12, cy - 3, (255, 188, 128))
    c.px(22, cy - 2, (255, 188, 128))
    c.rect(11, cy + 4, 15, 4, shell_lo)
    c.rect(15, cy + 5, 10, 2, (246, 198, 138))

    # Near-side legs drawn in front of lower carapace
    for i, lx in enumerate((8, 13, 18)):
        off = step if (i % 2 == 0) else -step
        if pose == "w2" and i % 2 == 0:
            off = -2
        elif pose == "w4" and i % 2 == 1:
            off = -2
        c.rect(lx, 24 + off // 2, 2, 6, shell)
        c.rect(lx, 30 + off // 2, 1, 3, shell_hi)

    # Woven Kelp Satchel carried cleanly across near hip (no stray gaps)
    c.line(11, cy - 3, 21, cy + 5, kelp, width=2)
    c.line(11, cy - 4, 20, cy + 4, kelp_hi, width=1)
    c.rect(12, cy + 1, 7, 6, kelp_lo)
    c.rect(13, cy + 1, 5, 3, kelp)
    c.px(16, cy + 4, (240, 206, 98))

    # 3/4 Claws with shaded inner pincer jaws (zero CLEAR cutouts!)
    snap_near = {"w1": -2, "w2": -1, "w3": 1, "w4": 0}.get(pose, 0)
    snap_far = {"w1": 1, "w2": -1, "w3": -2, "w4": 0}.get(pose, 0)
    c.oval(31, cy - 4 + snap_far, 4, 4, shell)
    if pose in ("stand", "idle", "w2", "w3"):
        c.rect(31, cy - 7 + snap_far, 2, 3, cleft)
        c.px(30, cy - 7 + snap_far, shell_hi)
        c.px(33, cy - 7 + snap_far, shell_hi)
    c.oval(8, cy - 1 + snap_near, 4, 4, shell_hi)
    if pose in ("stand", "idle", "w1", "w4"):
        c.rect(7, cy - 4 + snap_near, 2, 3, cleft)
        c.px(6, cy - 4 + snap_near, shell_hi)
        c.px(9, cy - 4 + snap_near, shell)

    # 3/4 shifted eyestalks & blinking eyes facing right
    eye_bob = -1 if pose in ("w2", "w4") else 0
    c.rect(17, cy - 11 + eye_bob, 2, 5 - eye_bob, shell)
    c.rect(23, cy - 11 + eye_bob, 2, 5 - eye_bob, shell)
    eye_mood = "closed" if pose == "idle" else "normal"
    c.eyes(17, 23, cy - 12 + eye_bob, iris=(28, 20, 38), mood=eye_mood, wide=True)
    c.line(19, cy + 1, 25, cy + 1, (70, 24, 16))
    return c.finish(OUTLINE_WARM)


def draw_seahorse_guard(variant=0, pose="stand"):
    """Seahorse guard with 4-frame banner flutter, segmented armor, and solid coiled tail spiral (34x38)."""
    c = Canvas(34, 38)
    bob = {"stand": 0, "idle": 1, "w1": -1, "w2": 0, "w3": 1, "w4": 0}.get(pose, 0)
    rib_dy = {"stand": 0, "idle": 0, "w1": -1, "w2": 0, "w3": 1, "w4": 0}.get(pose, 0)
    fin_fl = {"w1": 1, "w2": 0, "w3": -1, "w4": 0}.get(pose, 0)
    body = (76, 194, 168) if variant == 0 else (92, 126, 232)
    body_hi = shade(body, 1.24)
    body_lo = shade(body, 0.72)
    armor = (218, 178, 86) if variant == 0 else (186, 196, 218)

    sx = 26
    c.rect(sx, 2 + bob, 2, 32, (132, 86, 52))
    c.poly([(sx - 1, 5 + bob), (sx + 1, 0), (sx + 3, 5 + bob)], (232, 242, 255))
    c.poly([(sx + 2, 6 + bob), (sx + 7, 7 + bob + rib_dy), (sx + 5, 9 + bob + rib_dy), (sx + 2, 8 + bob)], (226, 54, 96))

    # Solid coiled tail with shaded inner dimple (no transparent hole!)
    c.oval(14, 29 + bob, 5, 4, body_lo)
    c.rect(13, 28 + bob, 2, 2, shade(body_lo, 0.65))
    c.oval(15, 21 + bob, 5, 7, body)
    c.rect(13, 17 + bob, 4, 8, body_hi)
    for ry in (18, 21, 24):
        c.line(12, ry + bob, 18, ry + bob, shade(armor, 0.85))
    c.poly([(10, 17 + bob), (4 - fin_fl, 20 + bob + fin_fl), (10, 24 + bob)], body_hi)

    c.oval(16, 11 + bob, 5, 4, body)
    c.rect(20, 11 + bob, 5, 2, body_hi)
    c.eyes(15, 18, 10 + bob, iris=(24, 24, 48), mood=("closed" if pose == "idle" else "normal"))
    c.rect(11, 5 + bob, 10, 4, armor)
    c.rect(12, 6 + bob, 6, 1, shade(armor, 1.25))
    c.poly([(12, 4 + bob), (16, 1 + bob), (20, 4 + bob)], (216, 52, 88))
    return c.finish(OUTLINE_SEA)


def draw_queen_crab(pose="stand"):
    """Queen Crab: 4-frame 3/4 King Crab monarch with billowing cloak, jewel crown & alternating claws (46x42)."""
    c = Canvas(46, 42)
    bob = {"stand": 0, "idle": 1, "w1": 0, "w2": -1, "w3": 0, "w4": 1}.get(pose, 0)
    flare = {"w1": 2, "w2": 1, "w3": 0, "w4": 1}.get(pose, 0)
    shell = (214, 65, 40)
    shell_hi = (252, 122, 78)
    shell_lo = (146, 30, 22)
    cape = (118, 22, 56)

    for i, lx in enumerate((12, 17, 27, 32)):
        leg_dy = -1 if ((i % 2 == 0 and pose == "w1") or (i % 2 == 1 and pose == "w3")) else 0
        c.rect(lx, 31 + bob + leg_dy, 3, 8, shell_lo)
    c.poly([(10, 18), (29, 18), (31 + flare, 36), (5 - flare, 36)], cape)
    c.rect(6 - flare, 33, 24 + flare * 2, 3, (245, 238, 232))
    c.oval(22, 20 + bob, 12, 9, shell)
    c.oval(19, 16 + bob, 8, 5, shell_hi)
    for sx, sy in ((11, 15), (15, 11), (31, 12), (34, 16), (11, 22), (34, 23)):
        c.poly([(sx - 2, sy + bob + 2), (sx, sy + bob - 3), (sx + 2, sy + bob + 2)], shell_lo)

    claw_near = {"w1": -2, "w2": 0, "w3": 2, "w4": 0}.get(pose, 0)
    claw_far = {"w1": 2, "w2": 0, "w3": -2, "w4": 0}.get(pose, 0)
    c.oval(39, 18 + claw_far, 5, 5, shell)
    c.oval(8, 22 + claw_near, 5, 5, shell_hi)

    for nx in range(18, 32, 3):
        c.rect(nx, 28 + bob, 2, 2, (246, 240, 210))
    c.oval(25, 30 + bob, 3, 3, (96, 232, 255))
    c.px(24, 29 + bob, (255, 255, 255))

    c.rect(18, 6 + bob, 14, 5, (244, 198, 72))
    for cx in (18, 24, 30):
        c.rect(cx, 3 + bob, 2, 3, (244, 198, 72))
    c.px(25, 8 + bob, (255, 70, 140))
    c.px(21, 8 + bob, (92, 232, 176))
    c.px(29, 8 + bob, (92, 232, 176))

    c.eyes(21, 28, 16 + bob, iris=(40, 12, 20), mood=("closed" if pose == "idle" else "angry"))
    return c.finish(OUTLINE_WARM)


def draw_sophie(pose="stand"):
    """Sophie: 4-frame skipping Clownfish Jester with solid hip seam & swinging cap bells (32x36)."""
    c = Canvas(32, 36)
    bob = {"stand": 0, "idle": 1, "w1": -1, "w2": -2, "w3": -1, "w4": 0}.get(pose, 0)
    bell_off = {"stand": 0, "idle": 0, "w1": 1, "w2": 0, "w3": -1, "w4": 0}.get(pose, 0)
    leg_off = {"w1": 2, "w2": 0, "w3": -2, "w4": 0}.get(pose, 0)
    orange = (255, 122, 42)
    orange_lo = (204, 78, 20)
    white = (250, 246, 240)

    c.rect(10, 23 + bob, 11, 5, (46, 36, 68))  # solid hip seam prevents gap during high skip
    c.rect(17 + leg_off // 2, 26 - leg_off // 2, 4, 8, (38, 30, 56))
    c.rect(11 - leg_off // 2, 26 + leg_off // 2, 4, 8, (54, 42, 78))
    c.rect(10, 15 + bob, 11, 11, orange)
    c.rect(15, 15 + bob, 4, 11, white)
    c.rect(20, 16 + bob - bell_off, 2, 6, orange_lo)
    c.rect(8, 16 + bob + bell_off, 3, 7, orange)
    c.rect(10, 14 + bob, 12, 3, (244, 212, 92))
    c.oval(16, 9 + bob, 6, 5, orange)
    c.rect(19, 6 + bob, 2, 7, white)
    c.eyes(14, 19, 8 + bob, iris=(28, 22, 44), mood=("closed" if pose == "idle" else "normal"))
    c.poly([(11, 6 + bob), (5, 1 + bob + bell_off), (15, 4 + bob)], (216, 48, 86))
    c.poly([(20, 6 + bob), (25, 2 + bob - bell_off), (17, 4 + bob)], (82, 194, 130))
    c.rect(4, 1 + bob + bell_off, 2, 2, (250, 216, 86))
    c.rect(24, 2 + bob - bell_off, 2, 2, (250, 216, 86))
    return c.finish(OUTLINE_WARM)


def draw_towtou(pose="stand"):
    """Tow-Tou: 4-frame paddling Leatherback turtle with seated cart, beret, scar, and brass fin (48x42)."""
    c = Canvas(48, 42)
    bob = {"stand": 0, "idle": 1, "w1": 0, "w2": -1, "w3": 0, "w4": 1}.get(pose, 0)
    fin_dy = {"stand": 0, "idle": 0, "w1": -2, "w2": 0, "w3": 2, "w4": 1}.get(pose, 0)
    hind_dx = {"w1": -2, "w2": 0, "w3": 2, "w4": 0}.get(pose, 0)
    shell = (42, 74, 80)
    shell_hi = (68, 114, 120)
    ridge = (146, 184, 176)
    skin = (72, 118, 108)

    # Flush-seated wooden passenger cart resting solidly into upper shell (eliminating pinhole!)
    c.rect(10, 6 + bob, 18, 12, (146, 96, 58))
    c.rect(11, 7 + bob, 16, 8, (178, 124, 78))
    c.rect(9, 6 + bob, 20, 2, (216, 180, 96))
    c.rect(7, 10 + bob, 3, 4, (255, 226, 116))
    c.rect(10, 16 + bob, 18, 4, (92, 62, 42))

    # Scute-patterned shell + deeply overlapped flippers
    c.oval(20, 24 + bob, 14, 9, shell)
    for ry in (19, 23, 27):
        c.line(9, ry + bob, 31, ry + bob, ridge)
    c.oval(18, 19 + bob, 10, 3, shell_hi)

    c.poly([(10, 28 + bob), (4 + hind_dx, 35 - bob), (15, 34 - bob)], skin)
    c.poly([(27, 27 + bob), (36, 36 + fin_dy), (24, 34 + bob)], (226, 182, 78))
    c.rect(27, 27 + bob, 5, 4, (150, 110, 46))
    c.px(32, 33 + fin_dy, (255, 236, 148))

    # Head + snowy beard + eye scar + tilted maroon beret
    c.oval(37, 21 + bob, 6, 5, skin)
    c.rect(35, 24 + bob, 6, 6, (238, 240, 245))
    c.eyes(35, 40, 19 + bob, iris=(24, 24, 40), mood=("closed" if pose == "idle" else "normal"))
    c.line(34, 17 + bob, 37, 22 + bob, (198, 62, 76))
    c.oval(36, 15 + bob, 6, 2, (196, 44, 72))
    c.px(33, 13 + bob, (238, 198, 92))
    return c.finish(OUTLINE_SEA)


def draw_genie(pose="stand"):
    """Magic Fire Genie: 4-frame blazing flame spirit with connected vortex & swaying crown (38x42)."""
    c = Canvas(38, 42)
    sway = {"stand": 0, "idle": 0, "w1": 2, "w2": 1, "w3": -2, "w4": -1}.get(pose, 0)
    flick = {"stand": 0, "idle": 1, "w1": 2, "w2": 0, "w3": -2, "w4": 0}.get(pose, 0)
    fire_out = (242, 92, 34)
    fire_mid = (255, 168, 54)
    fire_core = (255, 242, 148)

    c.poly([(19 + sway, 40), (11, 27), (27, 27)], fire_out)
    c.poly([(19 + sway // 2, 36), (14, 26), (24, 26)], fire_mid)
    c.poly([(9, 17), (29, 17), (24, 28), (14, 28)], fire_out)
    c.poly([(12, 18), (26, 18), (22, 27), (16, 27)], fire_mid)
    c.rect(13, 25, 12, 2, (238, 198, 86))
    c.rect(11, 15, 16, 4, fire_out)
    # Attached swirling shoulder/arm flames rooted solidly into roaring torso
    c.rect(5, 16 - sway // 2, 7, 8, fire_mid)
    c.rect(26, 16 + sway // 2, 7, 8, fire_mid)
    c.rect(5, 22 - sway // 2, 5, 2, (238, 198, 86))
    c.rect(28, 22 + sway // 2, 5, 2, (238, 198, 86))
    c.oval(19, 12, 5, 5, fire_mid)
    # Multi-tongue attached flame crown (no detached floating ember pixels!)
    c.poly([(12, 10), (15, 2 + flick // 2), (18, 7), (20, 1 - flick // 2), (24, 9), (26, 4 + flick // 2), (25, 10)], fire_core)
    c.rect(16, 11, 2, 2, (255, 255, 245))
    c.rect(20, 11, 2, 2, (255, 255, 245))
    return c.finish((120, 28, 18, 255))


# ---------------------------------------------------------------------------
# Expressive 48x48 Pixel-Art Bust Portraits
# ---------------------------------------------------------------------------

def draw_portrait(kind, expr="normal"):
    """48x48 hue-shifted & dark-outlined pixel bust portrait for Undertale-style dialogue."""
    bg_map = {
        "river": (30, 44, 84), "nadia": (58, 36, 82), "grandparent": (72, 50, 42), "mark": (44, 52, 70),
        "kenny": (20, 72, 78), "guard_a": (24, 64, 84), "guard_b": (28, 48, 96),
        "queen_crab": (82, 20, 44), "sophie": (68, 30, 58),
        "jack": (18, 44, 82), "jill": (26, 38, 80), "angelfish": (22, 52, 84), "stonefish": (28, 50, 64),
        "gill": (18, 58, 78), "scal": (20, 46, 82), "lady": (32, 52, 78), "towtou": (20, 48, 60),
        "angler": (24, 16, 48), "thaumiel": (16, 28, 44), "tenebris": (16, 14, 34), "genie": (46, 16, 26),
    }
    bg_color = bg_map.get(kind, (34, 24, 54))
    bg = Canvas(48, 48)
    bg.rect(0, 0, 48, 48, shade(bg_color, 0.78))
    bg.rect(1, 1, 46, 46, bg_color)
    bg.oval(24, 22, 19, 19, shade(bg_color, 1.15))
    bg.oval(22, 18, 13, 13, shade(bg_color, 1.28))

    # Foreground bust layer drawn cleanly then outlined & composited over vignette
    c = Canvas(48, 48)

    def bust(skin, shirt, hair, eye_col=(34, 24, 52), long_hair=False, specs=False, beard=False, floral=False):
        c.poly([(8, 47), (14, 34), (34, 34), (40, 47)], shirt)
        c.poly([(16, 34), (32, 34), (35, 47), (13, 47)], shade(shirt, 1.14))
        c.rect(14, 35, 5, 11, shade(shirt, 0.84))
        c.line(18, 37, 19, 44, shade(shirt, 0.72))
        c.line(30, 37, 29, 44, shade(shirt, 0.80))
        if floral:
            for fx, fy, fc in ((19, 40, (255, 206, 98)), (28, 39, (255, 136, 186)), (22, 43, (140, 236, 168))):
                c.px(fx, fy, fc); c.px(fx + 1, fy, shade(fc, 1.2))
        c.rect(20, 29, 8, 7, shade(skin, 0.80))
        c.rect(21, 31, 6, 3, shade(skin, 0.72))
        if long_hair:
            c.rect(10, 14, 28, 24, shade(hair, 0.76))
            c.rect(11, 16, 3, 21, shade(hair, 0.96))
            c.rect(34, 16, 3, 21, shade(hair, 0.70))
        # Sculpted face with subtle jawline shadow & cheek blush
        c.oval(24, 21, 10, 10, skin)
        c.oval(22, 18, 7, 6, shade(skin, 1.07))
        c.rect(16, 26, 16, 4, shade(skin, 0.88))
        c.rect(18, 28, 12, 3, shade(skin, 0.82))
        c.px(23, 24, shade(skin, 0.82))  # nose bridge shadow
        c.px(24, 24, shade(skin, 0.82))
        c.rect(15, 23, 3, 2, (238, 152, 170))
        c.rect(30, 23, 3, 2, (238, 152, 170))
        # Layered hair dome & specular highlight streak
        c.oval(24, 12, 11, 5, hair)
        c.rect(15, 10, 16, 3, shade(hair, 1.22))
        c.rect(18, 11, 10, 1, shade(hair, 1.36))
        c.rect(13, 12, 3, 9, shade(hair, 0.76))
        c.rect(32, 12, 3, 9, shade(hair, 0.92))
        for hx in (15, 19, 24, 29):
            c.rect(hx, 15, 3, 2, hair)
        if expr == "sick":
            c.rect(18, 21, 4, 1, (56, 38, 64)); c.px(19, 22, (74, 48, 84))
            c.rect(26, 21, 4, 1, (56, 38, 64)); c.px(27, 22, (74, 48, 84))
            c.rect(16, 24, 3, 2, (236, 146, 172))
            c.rect(29, 24, 3, 2, (236, 146, 172))
        else:
            for ex in (17, 26):
                c.rect(ex, 19, 5, 4, (248, 252, 255))
                c.rect(ex + 1, 20, 3, 3, eye_col)
                c.px(ex + 2, 21, shade(eye_col, 0.65))
                c.px(ex + 1, 20, (255, 255, 255))
            if expr == "shocked":
                c.rect(17, 16, 5, 1, (48, 28, 52)); c.rect(26, 16, 5, 1, (48, 28, 52))
            else:
                c.rect(18, 17, 4, 1, shade(hair, 0.72)); c.rect(26, 17, 4, 1, shade(hair, 0.72))
        if specs:
            c.rect(16, 18, 6, 6, (214, 242, 255, 115))
            c.rect(25, 18, 6, 6, (214, 242, 255, 115))
            c.rect(16, 18, 6, 1, (196, 160, 82)); c.rect(25, 18, 6, 1, (196, 160, 82))
            c.rect(22, 20, 3, 1, (196, 160, 82))
        if beard:
            c.oval(24, 29, 8, 5, hair)
            c.oval(24, 28, 6, 3, shade(hair, 1.15))
        if expr in ("cured", "happy"):
            c.line(21, 27, 24, 29, (110, 44, 58))
            c.line(24, 29, 27, 27, (110, 44, 58))
        elif expr == "shocked":
            c.oval(24, 28, 2, 2, (98, 36, 50))
        elif expr == "sick":
            c.line(22, 28, 26, 28, (130, 68, 86))
        else:
            c.rect(21, 27, 6, 1, (110, 48, 58))

    if kind == "river":
        bust((242, 196, 160), (46, 134, 232), (107, 68, 46))
        c.poly([(19, 34), (24, 38), (29, 34)], (242, 246, 252))
    elif kind == "nadia":
        bust((246, 204, 174), (118, 58, 168), (242, 206, 98), long_hair=True, floral=True)
    elif kind == "grandparent":
        bust((232, 184, 152), (156, 112, 74), (192, 198, 206), specs=True, beard=True)
    elif kind == "mark":
        bust((232, 184, 150), (74, 92, 116), (62, 42, 34))
    elif kind == "kenny":
        shell = (218, 78, 38)
        shell_hi = shade(shell, 1.24)
        shell_lo = shade(shell, 0.72)
        c.oval(24, 31, 14, 12, shell_lo)
        c.oval(24, 29, 13, 11, shell)
        c.oval(22, 24, 9, 6, shell_hi)
        c.oval(20, 22, 5, 3, shade(shell_hi, 1.15))
        # Kelp slingbag woven strap + brass buckle
        c.line(12, 23, 36, 41, (54, 136, 74), width=4)
        c.line(13, 23, 35, 40, (92, 196, 108), width=2)
        c.rect(22, 30, 4, 4, (238, 196, 86))
        # Sculpted eyestalks & glossy crab eyes (no flat sticker dots!)
        c.rect(17, 13, 3, 8, shell_lo); c.rect(17, 13, 2, 7, shell)
        c.rect(28, 13, 3, 8, shell_lo); c.rect(28, 13, 2, 7, shell)
        for ex in (16, 27):
            c.rect(ex, 8, 5, 5, (242, 248, 255))
            c.rect(ex + 1, 9, 3, 3, (28, 20, 42))
            c.px(ex + 1, 9, (255, 255, 255))
        # Framed claws on sides with lighter inner pinch edge
        c.oval(8, 26, 5, 5, shell_lo); c.oval(7, 25, 4, 4, shell_hi)
        c.oval(40, 26, 5, 5, shell_lo); c.oval(39, 25, 4, 4, shell_hi)
        c.rect(6, 22, 2, 4, shade(bg_color, 1.0)); c.rect(40, 22, 2, 4, CLEAR)
        c.px(6, 22, CLEAR); c.px(6, 23, CLEAR)
        if expr == "happy":
            c.line(19, 30, 24, 33, (56, 16, 14)); c.line(24, 33, 29, 30, (56, 16, 14))
        elif expr == "worried":
            c.line(19, 32, 29, 30, (56, 16, 14))
        else:
            c.rect(20, 31, 8, 1, (56, 16, 14))
    elif kind in ("guard_a", "guard_b"):
        col = (76, 194, 168) if kind == "guard_a" else (92, 126, 232)
        c.poly([(11, 47), (16, 30), (32, 30), (37, 47)], shade(col, 0.85))
        c.rect(21, 31, 6, 15, shade(col, 1.15))
        c.oval(24, 22, 9, 8, col)
        c.oval(22, 19, 6, 5, shade(col, 1.18))
        c.rect(31, 21, 9, 4, shade(col, 1.12))
        c.rect(15, 11, 18, 6, (218, 178, 86)); c.rect(16, 11, 16, 2, (246, 218, 128))
        c.poly([(17, 11), (24, 4), (31, 11)], (216, 52, 88))
        c.rect(22, 19, 4, 4, (250, 252, 255)); c.rect(23, 20, 2, 2, (24, 24, 48))
        c.px(23, 20, (255, 255, 255))
    elif kind == "queen_crab":
        shell = (232, 54, 46) if expr == "fuming" else (206, 58, 38)
        shell_hi = shade(shell, 1.24)
        shell_lo = shade(shell, 0.72)
        # Ermine velvet cape + collar fluff + ruby-pearl choker
        c.poly([(7, 47), (12, 31), (36, 31), (41, 47)], (104, 18, 52))
        c.poly([(12, 31), (36, 31), (39, 36), (9, 36)], (242, 234, 226))
        for nx in range(13, 35, 5):
            c.px(nx, 33, (46, 26, 40))
        c.oval(24, 25, 14, 11, shell_lo)
        c.oval(24, 24, 13, 10, shell)
        c.oval(22, 19, 9, 5, shell_hi)
        for sx, sy in ((11, 18), (15, 13), (33, 13), (37, 18)):
            c.poly([(sx - 2, sy + 2), (sx, sy - 4), (sx + 2, sy + 2)], shell_lo)
            c.px(sx, sy - 2, shell_hi)
        # Sculpted gold filigree crown
        c.rect(15, 8, 18, 5, (218, 162, 46))
        c.rect(16, 8, 16, 3, (246, 204, 82))
        for cx in (15, 22, 30):
            c.rect(cx, 3, 3, 5, (246, 204, 82))
            c.px(cx + 1, 3, (255, 240, 152))
        c.rect(23, 9, 2, 2, (255, 62, 132)); c.px(18, 10, (92, 236, 182)); c.px(29, 10, (92, 236, 182))
        for nx in range(16, 33, 3):
            c.rect(nx, 36, 2, 2, (248, 242, 215))
        c.oval(24, 39, 3, 3, (96, 238, 255)); c.px(23, 38, (255, 255, 255))
        c.eyes(18, 27, 21, iris=(40, 12, 20), mood="angry" if expr == "fuming" else "normal")
        if expr == "fuming":
            c.rect(8, 10, 3, 3, (255, 228, 234)); c.rect(37, 10, 3, 3, (255, 228, 234))
            c.rect(20, 29, 8, 3, (56, 10, 16))
        else:
            c.rect(20, 29, 8, 1, (64, 12, 18))
    elif kind == "sophie":
        c.poly([(9, 47), (15, 34), (33, 34), (39, 47)], (242, 112, 36))
        c.rect(20, 34, 8, 13, (250, 246, 240))
        c.oval(24, 22, 10, 9, (255, 126, 46))
        c.oval(22, 19, 6, 5, (255, 164, 84))
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
        crest_y = 2 if expr == "shocked" else 4
        c.poly([(15, 15), (23, crest_y), (28, 15)], (42, 84, 156))
        c.oval(23, 23, 9, 9, col); c.oval(21, 20, 5, 5, shade(col, 1.18))
        c.poly([(30, 22), (45, 25), (30, 27)], (188, 218, 244))
        c.line(31, 23, 42, 25, (232, 246, 255))
        if expr == "shocked":
            c.rect(16, 17, 5, 6, (252, 253, 255)); c.rect(24, 17, 5, 6, (252, 253, 255))
            c.rect(18, 19, 2, 2, (20, 28, 54)); c.rect(26, 19, 2, 2, (20, 28, 54))
            c.oval(24, 29, 2, 2, (24, 22, 46))
            c.rect(8, 10, 2, 6, (255, 224, 102)); c.rect(8, 18, 2, 2, (255, 224, 102))
        else:
            c.eyes(18, 25, 20, iris=(20, 28, 54), mood="normal")
            c.rect(21, 29, 6, 1, (24, 28, 52))
        if kind == "jack":
            c.rect(24, 19, 5, 5, (238, 202, 102))
            c.rect(25, 20, 3, 3, (176, 242, 255))
    elif kind == "towtou":
        c.poly([(8, 47), (14, 34), (34, 34), (40, 47)], (42, 74, 80))
        c.oval(24, 23, 10, 9, (72, 118, 108)); c.oval(22, 19, 6, 4, (96, 150, 136))
        c.oval(24, 31, 9, 6, (238, 240, 245))
        c.oval(23, 14, 11, 4, (196, 44, 72))
        c.eyes(18, 26, 21, iris=(24, 24, 40))
        c.line(17, 17, 22, 25, (198, 62, 76), width=2)
    elif kind == "thaumiel":
        c.poly([(8, 47), (15, 32), (33, 32), (40, 47)], (64, 78, 62))
        c.oval(24, 23, 10, 9, (92, 116, 134))
        c.line(23, 14, 30, 7, (56, 70, 96), width=2); c.line(30, 7, 36, 15, (56, 70, 96), width=2)
        c.oval(36, 17, 4, 4, (104, 244, 255)); c.oval(36, 17, 2, 2, (255, 255, 255))
        c.eyes(18, 26, 21, iris=(110, 244, 255))
    elif kind == "tenebris":
        c.poly([(10, 47), (16, 35), (32, 35), (38, 47)], (28, 26, 42))
        c.oval(24, 26, 8, 7, (62, 68, 96))
        c.rect(11, 18, 26, 3, (24, 22, 36))
        c.rect(16, 11, 16, 7, (34, 30, 50))
        c.rect(18, 24, 4, 2, (110, 240, 255)); c.rect(26, 24, 4, 2, (110, 240, 255))
    elif kind == "genie":
        c.poly([(10, 47), (16, 31), (32, 31), (38, 47)], (228, 78, 28))
        c.poly([(15, 33), (33, 33), (35, 47), (13, 47)], (255, 138, 48))
        c.oval(24, 22, 10, 9, (255, 168, 54)); c.oval(22, 19, 6, 5, (255, 222, 106))
        c.poly([(14, 16), (19, 5), (24, 13), (29, 4), (34, 16)], (255, 240, 142))
        c.rect(18, 20, 4, 3, (255, 255, 245)); c.rect(26, 20, 4, 3, (255, 255, 245))
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
        c.oval(24, 23, 10, 9, sk); c.oval(22, 20, 6, 5, shade(sk, 1.12))
        if kind == "angelfish":
            c.poly([(15, 15), (24, 3), (33, 15)], (245, 248, 255))
            c.rect(12, 16, 4, 12, (240, 244, 252)); c.rect(32, 16, 4, 12, (220, 228, 242))
            c.rect(18, 34, 12, 6, (255, 255, 255))
        elif kind == "stonefish":
            c.rect(15, 12, 18, 5, (190, 62, 74)); c.rect(10, 14, 7, 3, (150, 44, 56))
            c.rect(19, 27, 10, 1, (68, 54, 36))
        elif kind == "scal":
            c.oval(24, 15, 9, 4, (124, 152, 186)); c.rect(20, 28, 8, 1, (32, 54, 86))
        elif kind == "gill":
            c.poly([(19, 14), (24, 7), (29, 14)], (38, 134, 124))
            c.line(19, 27, 24, 29, (22, 68, 72)); c.line(24, 29, 29, 27, (22, 68, 72))
        elif kind == "lady":
            c.rect(12, 33, 24, 7, (244, 190, 162))
            c.oval(14, 16, 8, 4, (238, 242, 248)); c.oval(34, 16, 4, 4, (238, 242, 248))
            c.line(20, 27, 24, 29, (124, 42, 62)); c.line(24, 29, 28, 27, (124, 42, 62))
        elif kind == "angler":
            c.line(24, 14, 33, 9, (56, 70, 96), width=2)
            c.oval(35, 13, 3, 3, (255, 110, 210)); c.line(20, 28, 28, 27, (42, 22, 62))
        c.eyes(18, 26, 21, iris=(24, 24, 48))

    out_col = OUTLINE_ABYSS if kind in ("thaumiel", "tenebris", "angler") else (OUTLINE_WARM if kind in ("queen_crab", "kenny", "sophie", "genie", "nadia", "grandparent", "mark") else OUTLINE_SEA)
    fg = outline_image(c.im, out_col)
    res = bg.im.copy()
    res.alpha_composite(fg)
    return res


# ---------------------------------------------------------------------------
# Intro Storybook Plates (240x120 -> displayed at 3x = 720x360)
# ---------------------------------------------------------------------------

def _stamp(canvas, sprite_im, x, y, mirror=False):
    spr = sprite_im.transpose(Image.FLIP_LEFT_RIGHT) if mirror else sprite_im
    canvas.im.alpha_composite(spr, (int(x), int(y)))


def draw_intro_plates(cast_imgs=None, props_dir=None):
    plates = []
    river_spr = cast_imgs["river"] if cast_imgs else None
    mark_spr = cast_imgs["mark"] if cast_imgs else None
    tome_spr = None
    if props_dir and os.path.exists(os.path.join(props_dir, "tome.png")):
        tome_spr = Image.open(os.path.join(props_dir, "tome.png")).convert("RGBA")

    # Plate 0: Nadia resting in candlelit room, glowing sickbed & drifting runes
    c = Canvas(240, 120)
    c.rect(0, 0, 240, 120, (44, 28, 56))
    for wx in range(0, 240, 12):
        c.line(wx, 0, wx, 86, (38, 24, 50))
    c.rect(0, 86, 240, 34, (74, 48, 62))
    c.rect(0, 84, 240, 3, (124, 84, 58))
    c.line(0, 87, 240, 87, (46, 30, 38))
    # Starry bedroom window with warm oak mullions
    c.rect(22, 14, 48, 50, (92, 60, 44))
    c.rect(25, 17, 42, 44, (20, 26, 58))
    c.oval(36, 28, 5, 5, (250, 232, 162)); c.oval(38, 27, 4, 4, (20, 26, 58))
    c.px(56, 24, (220, 236, 255)); c.px(32, 48, (220, 236, 255)); c.px(58, 50, (250, 232, 162))
    c.rect(45, 17, 2, 44, (118, 78, 56)); c.rect(25, 38, 42, 2, (118, 78, 56))
    # Sculpted Bed + outlined Nadia sleeping silhouette + shaded Open Tome with drifting runes
    c.rect(86, 54, 106, 36, (72, 42, 34))
    c.rect(88, 56, 102, 34, (114, 68, 54))
    c.rect(85, 43, 10, 49, (84, 48, 36))
    c.rect(182, 53, 8, 39, (84, 48, 36))
    c.rect(96, 62, 86, 24, (148, 92, 186))
    c.rect(96, 62, 86, 5, (186, 124, 224))
    c.rect(95, 53, 24, 13, (246, 240, 230))
    c.rect(102, 46, 18, 15, (34, 20, 44))  # crisp dark outline around sleeping head
    c.rect(103, 47, 16, 13, (246, 204, 174))
    c.rect(101, 42, 19, 7, (34, 20, 44))
    c.rect(102, 43, 17, 6, (242, 206, 98))
    c.rect(102, 47, 5, 11, (208, 170, 74))
    c.rect(108, 54, 3, 1, (54, 34, 62)); c.rect(114, 54, 3, 1, (54, 34, 62))
    c.px(107, 56, (238, 146, 168)); c.px(116, 56, (238, 146, 168))
    c.rect(142, 52, 28, 17, (34, 20, 44))
    c.rect(144, 54, 24, 13, (246, 236, 214))
    c.rect(155, 54, 2, 13, (176, 152, 128))
    c.line(146, 57, 152, 57, (196, 172, 146)); c.line(159, 58, 165, 58, (196, 172, 146))
    for gx, gy in ((146, 42), (158, 36), (168, 44), (152, 28), (162, 24)):
        c.rect(gx, gy, 5, 2, (236, 152, 206, 210))
    plates.append(c.im)

    # Plate 1: Mark & River studying late by warm lamp using true 3/4 outlined sprites
    c = Canvas(240, 120)
    c.rect(0, 0, 240, 120, (34, 28, 50))
    c.rect(0, 90, 240, 30, (82, 54, 44))
    c.rect(0, 88, 240, 3, (124, 84, 58))
    c.line(0, 91, 240, 91, (48, 30, 36))
    c.oval(162, 54, 56, 42, (98, 66, 48))
    c.oval(164, 54, 34, 26, (162, 114, 60))
    # Doorway frame & Mark carrying crate in crisp 3/4 profile
    c.rect(38, 26, 42, 62, (24, 18, 36))
    c.rect(40, 28, 38, 60, (48, 34, 56))
    if mark_spr:
        _stamp(c, mark_spr, 43, 50)
    c.rect(63, 62, 15, 11, (148, 102, 64))
    c.line(63, 66, 78, 66, (96, 62, 38))
    # Oak study desk, glowing lamp, and crisp River studying
    if river_spr:
        _stamp(c, river_spr, 116, 40)
    c.rect(118, 68, 94, 22, (118, 78, 52))
    c.rect(118, 68, 94, 3, (152, 106, 74))
    c.rect(182, 46, 14, 11, (252, 222, 134))
    c.rect(188, 57, 2, 11, (90, 74, 86))
    c.rect(142, 61, 22, 7, (46, 134, 232))
    c.rect(144, 58, 18, 4, (246, 238, 218))
    plates.append(c.im)

    # Plate 2: Moonlit beach, crisp River & glowing Heirloom Tome washing ashore
    c = Canvas(240, 120)
    c.rect(0, 0, 240, 60, (14, 22, 54))
    c.oval(188, 22, 12, 12, (250, 240, 196))
    for sx, sy in ((28, 16), (68, 26), (112, 12), (144, 30), (214, 14), (92, 36)):
        c.rect(sx, sy, 2, 2, (240, 246, 255))
    c.rect(0, 56, 240, 24, (22, 68, 124))
    c.rect(0, 72, 240, 8, (110, 196, 224))
    c.rect(0, 80, 240, 40, (214, 186, 126))
    c.rect(0, 80, 240, 2, (240, 248, 255))
    if river_spr:
        _stamp(c, river_spr, 46, 56)
    c.oval(156, 82, 28, 14, (108, 240, 255, 130))
    if tome_spr:
        _stamp(c, tome_spr, 138, 58)
    plates.append(c.im)

    # Plate 3: Swirling tidal portal & Heirloom Tome pulling River into the deep
    c = Canvas(240, 120)
    c.rect(0, 0, 240, 120, (12, 20, 48))
    c.oval(120, 62, 90, 50, (24, 72, 124))
    c.oval(120, 62, 62, 34, (42, 134, 178))
    c.oval(120, 62, 36, 20, (122, 236, 246))
    if tome_spr:
        _stamp(c, tome_spr, 102, 52)
    if river_spr:
        _stamp(c, river_spr, 104, 10)
    for a in range(12):
        ang = a * math.pi / 6.0
        px = 120 + math.cos(ang) * (44 + (a % 3) * 14)
        py = 58 + math.sin(ang) * (22 + (a % 2) * 8)
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

    # 2. Kenny's Cozy Kelp Hut (96x80) - overhanging scalloped dome clipped cleanly above facade
    c = Canvas(96, 80)
    roof = (212, 102, 68); roof_hi = shade(roof, 1.24); roof_lo = shade(roof, 0.72)
    wall = (182, 132, 86); wall_lo = shade(wall, 0.78); wall_deep = shade(wall, 0.56)
    c.rect(62, 10, 12, 22, (132, 76, 72)); c.rect(60, 8, 16, 4, (164, 96, 88))
    c.rect(65, 2, 4, 4, (216, 244, 250, 170))
    # Overhanging shell-shingle dome drawn FIRST so timber walls mask bottom oval belly
    c.oval(48, 39, 42, 26, roof_lo)
    c.oval(48, 37, 41, 24, roof)
    c.oval(42, 27, 28, 13, roof_hi)
    c.rect(0, 43, 96, 37, CLEAR)  # clip dome cleanly into a side-view cap above walls
    for sy in (22, 30, 36):
        for sx in range(20 + (sy % 8), 76, 10):
            c.line(sx - 3, sy, sx + 3, sy, roof_lo)
            c.px(sx, sy + 1, roof_hi)
    # Recessed facade walls + dark soffit shadow + stone plinth base
    c.rect(14, 43, 68, 35, wall)
    for vx in range(22, 78, 10):
        c.line(vx, 47, vx, 75, wall_lo)
    c.rect(14, 43, 68, 6, wall_deep)
    c.rect(12, 74, 72, 4, (72, 64, 74))
    c.rect(12, 74, 72, 1, (108, 96, 104))
    # Crisp horizontal overhanging roof brim + kelp fascia garland above walls
    c.rect(7, 39, 82, 5, roof_lo)
    c.rect(8, 39, 80, 2, roof)
    # Arched timber door + warm brass knob + leaded glass windows
    c.rect(37, 51, 22, 27, (84, 48, 34))
    c.rect(39, 53, 18, 25, (108, 64, 44))
    c.rect(52, 65, 2, 3, (242, 198, 86))
    for wx in (21, 65):
        c.rect(wx, 54, 10, 10, (78, 48, 36))
        c.rect(wx + 1, 55, 8, 8, (255, 220, 124))
        c.line(wx + 4, 55, wx + 4, 63, (78, 48, 36))
        c.line(wx + 1, 59, wx + 9, 59, (78, 48, 36))
    for kx in range(10, 84, 8):
        c.rect(kx, 42, 6, 4, (54, 152, 82))
        c.px(kx + 2, 42, (110, 212, 134))
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "kenny_house.png"))

    # 3. Riddle Stone Monolith (36x52) with stepped stone plinth & glowing aqua veins
    c = Canvas(36, 52)
    c.poly([(6, 48), (8, 12), (18, 4), (28, 12), (30, 48)], (76, 92, 118))
    c.poly([(9, 46), (11, 13), (18, 6), (25, 13), (26, 46)], (124, 144, 174))
    c.line(11, 13, 18, 6, (176, 202, 232))
    c.rect(11, 15, 14, 25, (42, 54, 76))
    c.rect(12, 16, 12, 23, (54, 68, 94))
    for ry, rx in ((19, 15), (24, 17), (29, 14), (34, 16)):
        c.rect(rx, ry, 5, 2, (114, 248, 224))
    c.rect(14, 20, 2, 15, (72, 204, 196))
    c.rect(4, 46, 28, 4, (58, 72, 96))  # stepped base plinth
    c.rect(7, 43, 7, 4, (52, 150, 90)); c.rect(21, 44, 8, 3, (44, 134, 80))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "riddle_stone.png"))

    # 4. Throne Gate Archway (128x112) with projecting pediment eaves, masonry & plinth
    c = Canvas(128, 112)
    c.rect(16, 18, 96, 92, (62, 74, 98))
    c.rect(20, 22, 88, 86, (104, 120, 152))
    c.rect(16, 22, 96, 5, (48, 58, 80))  # shadow beneath overhanging pediment
    for by in range(30, 102, 12):
        c.line(20, by, 108, by, (70, 84, 112))
        off = 8 if (by // 12) % 2 else 0
        for bx in range(24 + off, 104, 16):
            c.line(bx, by, bx, min(106, by + 11), (70, 84, 112))
    c.rect(14, 105, 100, 5, (44, 54, 74))  # heavy threshold stone step
    c.poly([(8, 22), (64, 3), (120, 22)], (134, 150, 184))
    c.poly([(20, 21), (64, 8), (108, 21)], (162, 180, 212))
    c.rect(38, 42, 52, 68, (16, 20, 38))
    c.oval(64, 44, 26, 20, (16, 20, 38))
    c.rect(28, 26, 72, 5, (118, 232, 226))
    for px_col in (24, 98):
        for py_r in range(38, 96, 14):
            c.rect(px_col, py_r, 4, 6, (112, 236, 226))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "throne_gate.png"))

    # 5. Crab Castle Facade (180x140) with corbel eaves, brick depth & plinth foundation
    c = Canvas(180, 140)
    wall = (146, 52, 70); wall_hi = shade(wall, 1.24); wall_lo = shade(wall, 0.72); roof = (212, 60, 50)
    c.rect(20, 48, 140, 90, wall)
    c.rect(24, 52, 132, 84, wall_hi)
    for by in range(56, 134, 12):
        c.line(24, by, 156, by, wall)
    for mx in range(22, 156, 14):
        c.rect(mx, 44, 8, 6, wall_hi)
    for tx in (14, 134):
        c.rect(tx, 26, 32, 112, wall)
        c.rect(tx + 3, 28, 24, 108, wall_hi)
        c.rect(tx, 26, 32, 4, wall_lo)  # eave shadow under tower spire
        c.poly([(tx - 5, 26), (tx + 16, 5), (tx + 37, 26)], roof)
        c.rect(tx + 13, 38, 6, 12, (34, 18, 32))
    c.rect(64, 18, 52, 120, wall)
    c.rect(68, 20, 42, 116, wall_hi)
    c.rect(64, 18, 52, 5, wall_lo)
    c.poly([(58, 18), (90, 2), (122, 18)], roof)
    c.rect(12, 134, 156, 4, (84, 34, 48))  # castle plinth base
    c.rect(72, 84, 36, 54, (30, 16, 30))
    c.oval(90, 86, 18, 15, (30, 16, 30))
    c.rect(87, 36, 6, 18, (244, 198, 72))
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "crab_castle.png"))

    # 6. Shipwreck Hull & Anchor (96x64) with curved plank strakes & rigged sail
    c = Canvas(96, 64)
    wood = (118, 78, 50); wood_hi = shade(wood, 1.2); wood_lo = shade(wood, 0.68)
    c.poly([(6, 44), (18, 62), (76, 62), (90, 34), (62, 42)], wood)
    c.poly([(14, 47), (22, 59), (72, 59), (80, 41)], wood_lo)
    for hy in (49, 54, 58):
        c.line(16, hy, 76, hy - 3, wood_hi)
    for px in (28, 48, 66):
        c.rect(px, 50, 4, 4, (28, 34, 56))
    c.rect(42, 8, 5, 50, wood); c.rect(43, 8, 2, 46, wood_hi)
    c.line(24, 18, 64, 26, wood_lo, width=3)
    c.poly([(24, 20), (54, 26), (48, 40), (22, 34)], (214, 218, 204, 190))
    c.line(22, 34, 42, 48, (150, 138, 116))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "shipwreck.png"))

    # 7. Coral Houses (80x72) & Market Stall (84x68) with dome clipped above facade & soffit AO
    for idx, col in enumerate(((230, 114, 144), (86, 186, 202))):
        c = Canvas(80, 72)
        hi = shade(col, 1.22); lo = shade(col, 0.78); deep = shade(col, 0.56)
        c.oval(40, 29, 34, 22, lo)
        c.oval(40, 27, 33, 20, col)
        c.oval(35, 20, 22, 11, hi)
        c.rect(0, 33, 80, 39, CLEAR)  # clip dome cleanly into an overhanging coral roof cap
        for rx in (22, 34, 46, 58):
            c.line(rx, 13, rx, 32, lo)
        c.rect(14, 34, 52, 36, lo)
        c.rect(16, 36, 48, 32, col)
        c.rect(14, 34, 52, 5, deep)
        c.rect(12, 66, 56, 4, (62, 74, 92))
        c.rect(6, 30, 68, 4, lo)
        c.rect(7, 30, 66, 2, col)
        c.rect(31, 45, 18, 25, (72, 46, 56))
        c.rect(33, 47, 14, 23, (34, 24, 50))
        for wx in (20, 53):
            c.rect(wx, 44, 9, 9, (68, 42, 54))
            c.rect(wx + 1, 45, 7, 7, (255, 226, 138))
        c.finish(OUTLINE_SEA).save(os.path.join(out_dir, f"coral_house_{idx}.png"))

    c = Canvas(84, 68)
    c.rect(10, 26, 6, 40, (124, 82, 52)); c.rect(68, 26, 6, 40, (124, 82, 52))
    c.rect(8, 44, 68, 22, (164, 112, 68)); c.rect(10, 46, 64, 4, (194, 142, 92))
    for sy in (54, 60):
        c.line(10, sy, 74, sy, (118, 76, 46))
    for i in range(6):
        col = (238, 86, 128) if i % 2 == 0 else (248, 240, 222)
        c.rect(6 + i * 12, 13, 12, 15, col)
        c.rect(6 + i * 12, 26, 12, 3, shade(col, 0.8))
    c.rect(20, 37, 8, 7, (62, 166, 246)); c.rect(38, 36, 8, 8, (218, 92, 186)); c.rect(56, 37, 8, 7, (64, 196, 148))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "market_stall.png"))

    # 8. Tow-Tou's Scrapyard Garage (120x84) with corrugated overhang roof, soffit shadow & plinth
    c = Canvas(120, 84)
    c.rect(12, 18, 96, 64, (68, 78, 92))
    c.rect(16, 22, 88, 56, (96, 110, 126))
    for sx in range(18, 102, 7):
        c.line(sx, 22, sx, 78, (64, 74, 88))
    c.rect(14, 21, 92, 5, (42, 50, 62))  # roof overhang shadow
    c.rect(6, 12, 108, 9, (84, 96, 114))
    c.rect(6, 12, 108, 3, (128, 144, 168))  # bright metal roof trim
    c.rect(10, 78, 100, 4, (44, 52, 64))
    c.rect(32, 34, 56, 48, (22, 28, 38))
    for gy in range(36, 52, 4):
        c.rect(34, gy, 52, 2, (124, 136, 150))
    c.rect(74, 56, 10, 8, (255, 212, 96))
    c.rect(38, 22, 44, 10, (214, 142, 62)); c.rect(40, 24, 40, 6, (238, 174, 88))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "towtou_garage.png"))

    # 9. Whale Skeleton (144x68) - sculpted pixel skull + tooth pegs + ribbed vertebrae
    c = Canvas(144, 68)
    bone = (214, 220, 228); bone_hi = (242, 246, 252); bone_lo = (140, 150, 174)
    c.poly([(8, 44), (16, 26), (38, 22), (46, 34), (44, 49), (12, 49)], bone_lo)
    c.poly([(10, 42), (18, 28), (37, 25), (43, 35), (41, 46), (14, 46)], bone)
    c.poly([(16, 30), (32, 27), (36, 33), (20, 36)], bone_hi)
    c.oval(24, 36, 4, 4, (14, 16, 32))
    c.line(12, 46, 42, 46, (30, 28, 48))
    for tx in range(15, 39, 5):
        c.rect(tx, 46, 2, 2, bone_hi)
    for i, rx in enumerate(range(48, 132, 10)):
        r_len = max(10, 26 - i * 2)
        c.rect(rx, 33, 6, 6, bone_hi)
        c.rect(rx + 6, 35, 4, 3, bone_lo)
        c.poly([(rx + 1, 38), (rx + 5, 38), (rx + 7, 38 + r_len), (rx + 2, 38 + r_len)], bone)
        c.line(rx + 2, 39, rx + 3, 37 + r_len, bone_hi)
    c.finish(OUTLINE_ABYSS).save(os.path.join(out_dir, "whale_skeleton.png"))

    # Slum Shack (76x68) with protruding eaves, soffit shadow, lantern & stone base
    c = Canvas(76, 68)
    c.rect(10, 22, 56, 44, (44, 40, 62))
    for sy in (30, 38, 46, 54):
        c.line(12, sy, 64, sy, (32, 28, 48))
    c.rect(10, 22, 56, 4, (24, 20, 36))  # eave drop shadow
    c.rect(8, 62, 60, 4, (28, 24, 42))
    c.poly([(3, 24), (38, 5), (73, 24)], (64, 58, 88))
    c.poly([(10, 22), (38, 9), (66, 22)], (80, 72, 108))
    c.rect(28, 38, 16, 28, (22, 18, 34))
    c.rect(15, 36, 4, 6, (255, 196, 98))  # porch lantern glow
    c.rect(48, 34, 12, 12, (110, 232, 246))
    c.rect(53, 34, 2, 12, (44, 40, 62)); c.rect(48, 39, 12, 2, (44, 40, 62))
    c.finish(OUTLINE_ABYSS).save(os.path.join(out_dir, "slum_shack.png"))

    # Algae Temple (112x96) with cornice shadow & foundation step
    c = Canvas(112, 96)
    st = (54, 52, 88); st_hi = (92, 90, 142); st_lo = (34, 32, 58)
    c.rect(14, 28, 84, 66, st)
    c.rect(14, 28, 84, 5, st_lo)
    for by in range(38, 90, 12):
        c.line(14, by, 98, by, st_lo)
    for px in (18, 40, 64, 86):
        c.rect(px, 28, 8, 66, st_hi)
        c.rect(px + 2, 32, 2, 58, (118, 116, 178))
    c.poly([(6, 28), (56, 5), (106, 28)], st_hi)
    c.poly([(20, 26), (56, 11), (92, 26)], st)
    c.rect(48, 16, 16, 8, (82, 246, 206))
    c.rect(10, 90, 92, 4, st_lo)
    c.rect(42, 48, 28, 46, (14, 12, 28))
    c.oval(56, 70, 9, 11, (82, 246, 206)); c.oval(56, 70, 4, 5, (216, 255, 246))
    c.finish(OUTLINE_ABYSS).save(os.path.join(out_dir, "algae_temple.png"))

    # 9b. Elixir Tome (36x44) & Coral City Bespoke Reagents (28x28)
    c = Canvas(36, 44)
    c.rect(8, 5, 24, 35, (54, 22, 78))
    c.rect(10, 8, 21, 30, (248, 236, 208))
    for py in range(11, 35, 4):
        c.line(12, py, 29, py, (206, 184, 146))
    c.rect(5, 6, 22, 34, (90, 44, 128))
    c.rect(6, 7, 4, 32, (128, 72, 174))
    c.rect(11, 10, 13, 25, (76, 36, 112))
    c.rect(11, 10, 13, 1, (238, 194, 96)); c.rect(11, 34, 13, 1, (238, 194, 96))
    c.rect(11, 10, 1, 25, (238, 194, 96)); c.rect(23, 10, 1, 25, (238, 194, 96))
    c.poly([(17, 15), (22, 22), (17, 29), (12, 22)], (242, 198, 96))
    c.poly([(17, 18), (19, 22), (17, 26), (15, 22)], (104, 244, 255))
    c.rect(25, 19, 4, 6, (242, 198, 96)); c.rect(18, 37, 4, 5, (222, 68, 128))
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "tome.png"))

    # Blue Seashroom (28x28)
    c = Canvas(28, 28)
    c.rect(10, 15, 8, 10, (216, 234, 248))
    c.rect(11, 16, 3, 8, (242, 250, 255))
    c.oval(14, 11, 11, 6, (36, 146, 232))
    c.oval(12, 9, 8, 4, (86, 198, 255))
    for sx, sy in ((8, 9), (16, 7), (20, 10), (13, 12)):
        c.rect(sx, sy, 2, 2, (206, 252, 255))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "item_seashroom.png"))

    # Tentacle Anemone (28x28)
    c = Canvas(28, 28)
    c.rect(8, 19, 12, 6, (124, 48, 126))
    for tx, ty, h in ((7, 7, 13), (11, 4, 16), (15, 5, 15), (19, 8, 12)):
        c.rect(tx, ty, 3, h, (206, 76, 178))
        c.oval(tx + 1, ty, 2, 2, (255, 176, 238))
    c.finish(OUTLINE_WARM).save(os.path.join(out_dir, "item_anemone.png"))

    # True Seaberry (28x28 glowing azure berry bush)
    c = Canvas(28, 28)
    c.oval(14, 16, 10, 8, (42, 134, 98))
    c.oval(11, 12, 6, 5, (72, 178, 124))
    for bx, by in ((9, 12), (16, 10), (19, 16), (12, 18)):
        c.oval(bx, by, 3, 3, (48, 164, 255))
        c.px(bx - 1, by - 1, (226, 252, 255))
    c.finish(OUTLINE_SEA).save(os.path.join(out_dir, "item_seaberry.png"))

    # Decoy Kelp-Berry (28x28 dark violet/crimson thorned bush)
    c = Canvas(28, 28)
    c.oval(14, 16, 10, 8, (74, 52, 86))
    c.oval(12, 12, 6, 5, (108, 72, 114))
    for bx, by in ((9, 13), (17, 11), (14, 18)):
        c.oval(bx, by, 3, 3, (216, 48, 86))
        c.px(bx - 1, by - 1, (255, 164, 186))
    c.finish(OUTLINE_ABYSS).save(os.path.join(out_dir, "item_decoy.png"))

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

    def _assert_clean(im, tag):
        w, h = im.size
        px = im.load()
        bg = [[False] * w for _ in range(h)]
        q = [(x, 0) for x in range(w)] + [(x, h - 1) for x in range(w)] + [(0, y) for y in range(h)] + [(w - 1, y) for y in range(h)]
        head = 0
        for sx, sy in q:
            if px[sx, sy][3] == 0 and not bg[sy][sx]:
                bg[sy][sx] = True
            else:
                continue
        stack = [(x, y) for y in range(h) for x in range(w) if bg[y][x]]
        while stack:
            cx, cy = stack.pop()
            for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
                nx, ny = cx + dx, cy + dy
                if 0 <= nx < w and 0 <= ny < h and not bg[ny][nx] and px[nx, ny][3] == 0:
                    bg[ny][nx] = True
                    stack.append((nx, ny))
        holes = [(x, y) for y in range(h) for x in range(w) if px[x, y][3] == 0 and not bg[y][x]]
        seen = [[False] * w for _ in range(h)]
        comps = 0
        for y in range(h):
            for x in range(w):
                if px[x, y][3] > 0 and not seen[y][x]:
                    comps += 1
                    st = [(x, y)]
                    seen[y][x] = True
                    while st:
                        cx, cy = st.pop()
                        for dx in (-1, 0, 1):
                            for dy in (-1, 0, 1):
                                if dx == 0 and dy == 0:
                                    continue
                                nx, ny = cx + dx, cy + dy
                                if 0 <= nx < w and 0 <= ny < h and px[nx, ny][3] > 0 and not seen[ny][nx]:
                                    seen[ny][nx] = True
                                    st.append((nx, ny))
        assert not holes and comps == 1, f"{tag}: holes={holes} comps={comps}"

    cast_imgs = {}
    poses_all = ("stand", "idle", "w1", "w2", "w3", "w4")
    for cid, cfg in humanoid_specs.items():
        for pose in poses_all:
            im = draw_humanoid(cfg, pose)
            if pose == "stand":
                cast_imgs[cid] = im
            fn = cid if pose == "stand" else f"{cid}_{pose}"
            _assert_clean(im, fn)
            im.save(os.path.join(char_dir, f"{fn}.png"))
        make_shadow(draw_humanoid(cfg, "stand")).save(os.path.join(char_dir, f"{cid}_shadow.png"))

    # River swimming 4-frame cycle
    for idx in (1, 2, 3, 4):
        sim = draw_river_swim(f"swim{idx}")
        _assert_clean(sim, f"river_swim{idx}")
        sim.save(os.path.join(char_dir, f"river_swim{idx}.png"))

    # Bespoke non-humanoid cast (+ 4-frame action & idle breathing)
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
        for pose in poses_all:
            im = fn(pose)
            out_name = cid if pose == "stand" else f"{cid}_{pose}"
            _assert_clean(im, out_name)
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

    # Biome props first so Intro plates can stamp heirloom tome cleanly
    draw_props(props_dir)

    # Intro plates
    for idx, plate in enumerate(draw_intro_plates(cast_imgs, props_dir)):
        plate.save(os.path.join(intro_dir, f"plate_{idx}.png"))
    print(f"pixel_cast: {len(humanoid_specs) + len(custom_fns)} characters, portraits, 4 intro plates, props emitted")
