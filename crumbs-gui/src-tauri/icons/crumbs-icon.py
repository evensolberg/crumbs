#!/usr/bin/env python3
"""Generate the crumbs app icon — 1024×1024 PNG."""

from PIL import Image, ImageDraw, ImageFilter
import os

SIZE = 1024
OUT  = os.path.join(os.path.dirname(__file__), "icon-1024.png")

BG       = (8,   8,  28)
SURFACE  = (12,  20,  50)
TEAL     = (0,  240, 185)
TEAL_DIM = (0,  190, 145)
WHITE    = (245, 245, 255)
GOLD     = (255, 210,  60)

img  = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
draw = ImageDraw.Draw(img, "RGBA")

CARD_R = 200

# ── Teal glow behind card ──────────────────────────────────────────────────
glow_img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
gd = ImageDraw.Draw(glow_img, "RGBA")
gd.rounded_rectangle([80, 80, SIZE-80, SIZE-80], radius=CARD_R, fill=(0, 220, 170, 90))
glow_img = glow_img.filter(ImageFilter.GaussianBlur(40))
img.alpha_composite(glow_img)

# ── Drop shadow ────────────────────────────────────────────────────────────
shadow_img = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
sd = ImageDraw.Draw(shadow_img, "RGBA")
sd.rounded_rectangle([44, 60, SIZE-44, SIZE-28], radius=CARD_R, fill=(4, 4, 12, 180))
shadow_img = shadow_img.filter(ImageFilter.GaussianBlur(22))
img.alpha_composite(shadow_img)

# ── Main card ─────────────────────────────────────────────────────────────
draw.rounded_rectangle([32, 32, SIZE-32, SIZE-32], radius=CARD_R, fill=SURFACE)

# ── Subtle top highlight strip ────────────────────────────────────────────
for i, y in enumerate(range(33, 90)):
    a = int(22 * (1 - i / 57))
    draw.line([(33, y), (SIZE-33, y)], fill=(255, 255, 255, a))

# ── Teal top accent capsule ───────────────────────────────────────────────
draw.rounded_rectangle(
    [SIZE//2 - 110, 68, SIZE//2 + 110, 82],
    radius=7, fill=(*TEAL, 255)
)

# ── Checklist rows ────────────────────────────────────────────────────────
ROW_X  = 140
ROW_Y0 = 240
ROW_H  = 122
ROW_W  = SIZE - 280
CHECK_R = 30

rows = [
    (True,  True),   # checked, full opacity
    (True,  False),  # checked, slightly dim
    (False, True),   # open
    (False, False),  # open, dim
]

label_widths = [340, 280, 370, 220]

for i, (checked, bright) in enumerate(rows):
    y  = ROW_Y0 + i * ROW_H
    cy = y + ROW_H // 2

    # Row pill background
    if checked:
        fill = (*TEAL, 55 if bright else 32)
    else:
        fill = (255, 255, 255, 28 if bright else 14)
    draw.rounded_rectangle([ROW_X, y+12, ROW_X+ROW_W, y+ROW_H-12],
                           radius=16, fill=fill)

    # Check indicator circle
    cx = ROW_X + 52
    if checked:
        a = 255 if bright else 200
        draw.ellipse([cx-CHECK_R, cy-CHECK_R, cx+CHECK_R, cy+CHECK_R],
                     fill=(*TEAL, a))
        # Crisp checkmark
        pts = [(cx-15, cy+3), (cx-4, cy+15), (cx+17, cy-13)]
        draw.line([pts[0], pts[1]], fill=(*SURFACE, 255), width=7)
        draw.line([pts[1], pts[2]], fill=(*SURFACE, 255), width=7)
    else:
        a = 170 if bright else 100
        draw.ellipse([cx-CHECK_R, cy-CHECK_R, cx+CHECK_R, cy+CHECK_R],
                     outline=(*TEAL, a), width=4)

    # Label bar
    lx  = ROW_X + 104
    lw  = label_widths[i]
    lh  = 11
    la  = (200 if bright else 130)
    lc  = (*TEAL, la) if checked else (*WHITE, la)
    draw.rounded_rectangle([lx, cy-lh, lx+lw, cy+lh], radius=lh, fill=lc)

    # Trailing breadcrumb dot
    dx = ROW_X + ROW_W - 38
    dr = 8 if bright else 6
    draw.ellipse([dx-dr, cy-dr, dx+dr, cy+dr],
                 fill=(*TEAL, 200 if checked else 70))

# ── Breadcrumb trail ──────────────────────────────────────────────────────
trail_pts = [
    (200, 748), (310, 720), (430, 745),
    (550, 715), (670, 738), (790, 712), (860, 730),
]

# Dashed connecting lines
for i in range(len(trail_pts)-1):
    x1,y1 = trail_pts[i]
    x2,y2 = trail_pts[i+1]
    steps = 7
    for s in range(steps):
        if s % 2 == 1:
            continue
        t0 = s / steps
        t1 = (s + 0.5) / steps
        px0 = int(x1 + (x2-x1)*t0)
        py0 = int(y1 + (y2-y1)*t0)
        px1 = int(x1 + (x2-x1)*t1)
        py1 = int(y1 + (y2-y1)*t1)
        draw.line([(px0,py0),(px1,py1)], fill=(*TEAL_DIM, 110), width=4)

# Trail dots — varying sizes for depth
trail_sizes = [8, 11, 8, 13, 8, 11, 9]
for (x, y), r in zip(trail_pts, trail_sizes):
    draw.ellipse([x-r, y-r, x+r, y+r], fill=(*TEAL_DIM, 180))

# ── Three bottom crumb dots ───────────────────────────────────────────────
dot_y = 836
for dx, sz, col in [(-64, 11, TEAL), (0, 16, GOLD), (64, 11, TEAL)]:
    x = SIZE//2 + dx
    draw.ellipse([x-sz, dot_y-sz, x+sz, dot_y+sz], fill=(*col, 255))

# ── Save ──────────────────────────────────────────────────────────────────
img.save(OUT, "PNG")
print(f"Saved: {OUT}")
