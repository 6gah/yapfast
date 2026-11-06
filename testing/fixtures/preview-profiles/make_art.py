#!/usr/bin/env python3
"""Draws the preview's fake profile pictures: original coloured shapes, nothing
of Discord's. Run it here to write them again; it checks each file it writes.

PIL draws the frames. Animated PNGs are written by hand so that their acTL says
exactly how many times they play and every frame stays in the file, even a
wholly clear one, which PIL would merge into the frame before.
"""

import io
import math
import struct
import zlib
from pathlib import Path

from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
CLEAR = (0, 0, 0, 0)
FOREVER = 0
ONCE = 1


def chunk(kind, body):
    tagged = kind + body
    return struct.pack(">I", len(body)) + tagged + struct.pack(">I", zlib.crc32(tagged))


def write_apng(name, frames, delay_ms, plays):
    """Each frame replaces the whole canvas, shown for delay_ms."""
    width, height = frames[0].size
    out = [b"\x89PNG\r\n\x1a\n"]
    out.append(chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)))
    out.append(chunk(b"acTL", struct.pack(">II", len(frames), plays)))
    sequence = 0
    for index, frame in enumerate(frames):
        # dispose_op none, blend_op source.
        control = struct.pack(">IIIIIHHBB", sequence, width, height, 0, 0, delay_ms, 1000, 0, 0)
        out.append(chunk(b"fcTL", control))
        sequence += 1
        rgba = frame.convert("RGBA").tobytes()
        rows = b"".join(
            b"\0" + rgba[row * width * 4 : (row + 1) * width * 4] for row in range(height)
        )
        packed = zlib.compress(rows, 9)
        if index == 0:
            out.append(chunk(b"IDAT", packed))
        else:
            out.append(chunk(b"fdAT", struct.pack(">I", sequence) + packed))
            sequence += 1
    out.append(chunk(b"IEND", b""))
    data = b"".join(out)
    (HERE / name).write_bytes(data)
    check_apng(name, data, len(frames), plays)


def check_apng(name, data, frames, plays):
    at = data.index(b"acTL") + 4
    assert struct.unpack(">II", data[at : at + 8]) == (frames, plays), name
    with Image.open(io.BytesIO(data)) as opened:
        assert opened.n_frames == frames, name
        assert opened.info.get("loop", FOREVER) == plays, name


def canvas(size, background=CLEAR):
    image = Image.new("RGBA", size, background)
    return image, ImageDraw.Draw(image)


def star(draw, x, y, radius, fill):
    points = []
    for point in range(8):
        reach = radius if point % 2 == 0 else radius * 0.3
        angle = point * math.pi / 4
        points.append((x + reach * math.cos(angle), y + reach * math.sin(angle)))
    draw.polygon(points, fill=fill)


def effect_burst():
    """Plays once and ends clear, so repeating or sticking shows at a glance."""
    frames = []
    count = 10
    for index in range(count - 1):
        image, draw = canvas((112, 220))
        grown = (index + 1) / (count - 1)
        alpha = int(255 * (1 - grown * 0.8))
        for spoke in range(12):
            angle = spoke * math.tau / 12
            x = 56 + 52 * grown * math.cos(angle)
            y = 60 + 52 * grown * math.sin(angle)
            star(draw, x, y, 4 + 5 * grown, (255, 120 + spoke * 10, 40, alpha))
        frames.append(image)
    frames.append(canvas((112, 220))[0])
    write_apng("effect-burst.png", frames, 80, ONCE)


def effect_stars():
    spots = [(18, 30), (92, 22), (40, 90), (84, 120), (24, 170), (70, 196)]
    frames = []
    for index in range(10):
        image, draw = canvas((112, 220))
        for at, (x, y) in enumerate(spots):
            shine = math.sin((index / 10 + at / 6) * math.tau) * 0.5 + 0.5
            star(draw, x, y, 3 + 5 * shine, (255, 240, 130, int(90 + 165 * shine)))
        frames.append(image)
    write_apng("effect-stars.png", frames, 200, FOREVER)


def effect_loop():
    frames = []
    for index in range(8):
        image, draw = canvas((112, 220))
        for bubble in range(6):
            x = 12 + bubble * 18
            y = 220 - ((index * 27.5 + bubble * 37) % 220)
            draw.ellipse((x - 6, y - 6, x + 6, y + 6), outline=(90, 240, 180, 230), width=2)
        frames.append(image)
    write_apng("effect-loop.png", frames, 125, FOREVER)


def effect_once():
    frames = []
    for index in range(10):
        image, draw = canvas((112, 220))
        y = index * 24
        draw.rectangle((0, y, 111, y + 14), fill=(90, 150, 255, 170))
        frames.append(image)
    write_apng("effect-once.png", frames, 120, ONCE)


def avatars():
    looks = [
        ((224, 90, 90), "circle"),
        ((80, 170, 110), "square"),
        ((80, 120, 220), "triangle"),
        ((215, 165, 60), "diamond"),
        ((160, 95, 205), "circle"),
        ((60, 170, 180), "square"),
    ]
    for number, (colour, shape) in enumerate(looks, start=1):
        image, draw = canvas((64, 64), colour + (255,))
        ink = (255, 255, 255, 255)
        if shape == "circle":
            draw.ellipse((16, 16, 48, 48), fill=ink)
        elif shape == "square":
            draw.rectangle((18, 18, 46, 46), fill=ink)
        elif shape == "triangle":
            draw.polygon([(32, 14), (50, 48), (14, 48)], fill=ink)
        else:
            draw.polygon([(32, 12), (52, 32), (32, 52), (12, 32)], fill=ink)
        image.save(HERE / f"avatar-{number}.png", optimize=True)


def moving_avatar():
    """An avatar as Discord serves a moving one: a still, and the same as a GIF."""
    frames = []
    for index in range(8):
        image, draw = canvas((64, 64), (40, 50, 90, 255))
        turned = index * 45
        draw.pieslice((8, 8, 56, 56), turned, turned + 120, fill=(255, 200, 70, 255))
        draw.pieslice((8, 8, 56, 56), turned + 180, turned + 300, fill=(240, 90, 150, 255))
        frames.append(image.convert("RGB"))
    frames[0].save(HERE / "a_spin.png", optimize=True)
    frames[0].save(
        HERE / "a_spin.gif", save_all=True, append_images=frames[1:], duration=100, loop=0
    )


def decoration():
    frames = []
    for index in range(10):
        image, draw = canvas((120, 120))
        draw.ellipse((6, 6, 114, 114), outline=(120, 220, 255, 255), width=5)
        for dot in range(3):
            angle = (index / 10 + dot / 3) * math.tau
            x = 60 + 54 * math.cos(angle)
            y = 60 + 54 * math.sin(angle)
            draw.ellipse((x - 7, y - 7, x + 7, y + 7), fill=(255, 110, 180, 255))
        frames.append(image)
    write_apng("decoration-ring.png", frames, 100, FOREVER)


def nameplate():
    image, draw = canvas((224, 42))
    for x in range(0, 224, 2):
        draw.line((x, 0, x, 41), fill=(70, 200, 160, int(x * 200 / 224)), width=2)
    for x in (120, 160, 200):
        star(draw, x, 21, 11, (255, 255, 255, 230))
    image.save(HERE / "nameplate-still.png", optimize=True)


def banner():
    frames = []
    for index in range(8):
        image, draw = canvas((150, 60), (35, 45, 80, 255))
        for stripe in range(-1, 8):
            x = stripe * 24 + index * 3
            draw.polygon(
                [(x, 60), (x + 12, 60), (x + 42, 0), (x + 30, 0)],
                fill=(235, 130, 70, 255),
            )
        frames.append(image.convert("RGB"))
    frames[0].save(
        HERE / "banner-stripes.gif", save_all=True, append_images=frames[1:], duration=125, loop=0
    )


def badges():
    colours = [(240, 180, 60), (90, 200, 120), (100, 150, 240), (230, 100, 120)]
    for number, colour in enumerate(colours, start=1):
        image, draw = canvas((22, 22))
        star(draw, 11, 11, 10, colour + (255,))
        image.save(HERE / f"badge-{number}.png", optimize=True)
    frames = []
    for index in range(6):
        image, draw = canvas((22, 22))
        draw.ellipse((2, 2, 19, 19), fill=(170, 110, 240, 255))
        star(draw, 11, 11, 2 + index, (255, 255, 255, 255))
        frames.append(image)
    write_apng("badge-moving.png", frames, 150, FOREVER)


def tag_and_emoji():
    image, draw = canvas((24, 24))
    draw.rounded_rectangle((1, 1, 22, 22), 5, fill=(240, 120, 60, 255))
    draw.rectangle((7, 7, 16, 16), fill=(255, 255, 255, 255))
    image.save(HERE / "tag.png", optimize=True)
    image, draw = canvas((44, 44))
    draw.ellipse((3, 3, 40, 40), fill=(255, 215, 80, 255))
    draw.ellipse((13, 14, 19, 20), fill=(50, 40, 40, 255))
    draw.ellipse((25, 14, 31, 20), fill=(50, 40, 40, 255))
    draw.arc((12, 14, 32, 34), 20, 160, fill=(50, 40, 40, 255), width=3)
    image.save(HERE / "emoji-status.png", optimize=True)


def showcase():
    for name, colour in (("game-1", (90, 130, 200)), ("game-2", (200, 110, 90))):
        image, draw = canvas((64, 64), colour + (255,))
        draw.rectangle((12, 24, 52, 44), fill=(255, 255, 255, 255))
        image.save(HERE / f"{name}.png", optimize=True)
    image, draw = canvas((128, 128), (70, 60, 110, 255))
    star(draw, 64, 64, 48, (255, 200, 120, 255))
    image.save(HERE / "wish-1.png", optimize=True)


if __name__ == "__main__":
    effect_burst()
    effect_stars()
    effect_loop()
    effect_once()
    avatars()
    moving_avatar()
    decoration()
    nameplate()
    banner()
    badges()
    tag_and_emoji()
    showcase()
