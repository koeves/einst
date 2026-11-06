#!/usr/bin/env python3
"""Einst's app icon, from the in-app logo: the "einst." wordmark (Liberation
Sans Bold, the theme's WordmarkStyle: ChassisText on the black header, 0.88
horizontal scale) in a black rounded square with the chassis' light rim.
Small sizes (16 and 32 px, and their @2x) show "e." only, so it stays legible.

Writes AppIcon.png (1024 px) and, on macOS, AppIcon.icns (through `iconutil`)
into both UI sets, assets/ui and assets/ui-v1 (the app bundle takes ui-v1's). Needs Pillow (`pip install pillow`).

    python3 scripts/icon.py
"""
import os
import subprocess
import tempfile

from PIL import Image, ImageDraw, ImageFont

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
OUTS = [os.path.join(ROOT, "assets", "ui"), os.path.join(ROOT, "assets", "ui-v1")]
FONT = os.path.join(OUTS[0], "Fonts", "Liberation Sans Bold.ttf")

# scripts/uigen/theme.py: the header's black, ChassisText, the chassis rim.
BG = (10, 11, 13)
TEXT = (236, 236, 236)
RIM = (223, 225, 224)

SS = 4  # supersampling


def icon(n, word):
    """An n×n icon showing `word`."""
    s = n * SS
    k = s / 1024.0
    im = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    # macOS grid: an 824 px body at 100 px, corner radius 185.
    body = [100 * k, 100 * k, 924 * k, 924 * k]
    d.rounded_rectangle(body, radius=185 * k, fill=BG + (255,))
    # The light rim, inset like the in-app bezel.
    inset, width = 46 * k, max(2.0, 14 * k)
    d.rounded_rectangle([body[0] + inset, body[1] + inset, body[2] - inset, body[3] - inset], radius=140 * k, outline=RIM + (255,), width=round(width))
    # The wordmark: fitted to the rim's inner width, scaled 0.88 horizontally.
    inner = (824 - 2 * 46 - 2 * 60) * k
    size = 400
    font = ImageFont.truetype(FONT, size)
    l, t, r, b = font.getbbox(word)
    scale = min(inner / ((r - l) * 0.88), (430 if len(word) <= 2 else 300) * k / (b - t))
    font = ImageFont.truetype(FONT, max(1, round(size * scale)))
    l, t, r, b = font.getbbox(word)
    text = Image.new("RGBA", (r - l + 4, b - t + 4), (0, 0, 0, 0))
    ImageDraw.Draw(text).text((-l + 2, -t + 2), word, font=font, fill=TEXT + (255,))
    text = text.resize((max(1, round(text.width * 0.88)), text.height), Image.LANCZOS)
    im.alpha_composite(text, (round((s - text.width) / 2), round((s - text.height) / 2 + 12 * k)))
    return im.resize((n, n), Image.LANCZOS)


def for_size(n):
    return icon(n, "e." if n <= 64 else "einst.")


def main():
    with tempfile.TemporaryDirectory() as tmp:
        iconset = os.path.join(tmp, "AppIcon.iconset")
        os.makedirs(iconset)
        for size in (16, 32, 128, 256, 512):
            for scale in (1, 2):
                name = f"icon_{size}x{size}{'@2x' if scale == 2 else ''}.png"
                # Small sizes keep "e." also at @2x (the same point size).
                icon(size * scale, "e." if size <= 32 else "einst.").save(os.path.join(iconset, name))
        for out in OUTS:
            os.makedirs(out, exist_ok=True)
            for_size(1024).save(os.path.join(out, "AppIcon.png"))
            subprocess.run(["iconutil", "-c", "icns", iconset, "-o", os.path.join(out, "AppIcon.icns")], check=True)


if __name__ == "__main__":
    main()
