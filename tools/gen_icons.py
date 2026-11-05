#!/usr/bin/env python3
"""Генерация иконок приложения Fontlord (PNG, без внешних зависимостей).

Использование:  python3 tools/gen_icons.py
Результат:      src-tauri/icons/{32x32,128x128,128x128@2x,icon}.png
"""

from __future__ import annotations

import math
import os
import struct
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICONS = os.path.join(ROOT, "src-tauri", "icons")

BG_TOP = (26, 30, 41)
BG_BOTTOM = (14, 16, 23)
ACCENT = (120, 190, 255)
ACCENT_2 = (255, 170, 90)


def sdf_rounded_rect(px: float, py: float, hw: float, hh: float, r: float) -> float:
    qx = abs(px) - (hw - r)
    qy = abs(py) - (hh - r)
    ax = max(qx, 0.0)
    ay = max(qy, 0.0)
    return math.hypot(ax, ay) + min(max(qx, qy), 0.0) - r


def sdf_segment(px, py, ax, ay, bx, by, r):
    pax, pay = px - ax, py - ay
    bax, bay = bx - ax, by - ay
    denom = bax * bax + bay * bay
    h = 0.0 if denom == 0 else max(0.0, min(1.0, (pax * bax + pay * bay) / denom))
    dx = pax - bax * h
    dy = pay - bay * h
    return math.hypot(dx, dy) - r


def blend(dst, src, alpha):
    return tuple(int(round(d + (s - d) * alpha)) for d, s in zip(dst, src))


def render(size: int) -> bytes:
    rows = []
    s = size / 512.0
    for y in range(size):
        row = bytearray()
        for x in range(size):
            # фон: скруглённый прямоугольник с вертикальным градиентом
            bg_d = sdf_rounded_rect(x - size / 2, y - size / 2, size * 0.46, size * 0.46, size * 0.22)
            bg_a = clamp(0.5 - bg_d)
            t = y / max(1, size - 1)
            color = blend(BG_TOP, BG_BOTTOM, t)

            # основной мазок кистью (тело буквы «A»)
            d1 = sdf_segment(x, y, size * 0.30, size * 0.78, size * 0.52, size * 0.20, size * 0.075)
            d2 = sdf_segment(x, y, size * 0.52, size * 0.20, size * 0.70, size * 0.78, size * 0.075)
            d3 = sdf_segment(x, y, size * 0.40, size * 0.58, size * 0.62, size * 0.58, size * 0.055)
            stroke_a = clamp(0.5 - min(d1, d2, d3))
            color = blend(color, ACCENT, stroke_a)

            # «капля» наконечника
            dc = math.hypot(x - size * 0.74, y - size * 0.80) - size * 0.075
            color = blend(color, ACCENT_2, clamp(0.5 - dc))

            row += bytes((color[0], color[1], color[2], int(round(bg_a * 255))))
        # каждая строка скана начинается с байта фильтра (0 = None)
        rows.append(b"\x00" + bytes(row))
    return b"".join(rows)


def clamp(v: float) -> float:
    return max(0.0, min(1.0, v))


def write_png(path: str, size: int) -> None:
    raw = render(size)
    sig = b"\x89PNG\r\n\x1a\n"

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    ihdr = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    idat = zlib.compress(raw, 9)
    with open(path, "wb") as fh:
        fh.write(sig + chunk(b"IHDR", ihdr) + chunk(b"IDAT", idat) + chunk(b"IEND", b""))


def main() -> None:
    os.makedirs(ICONS, exist_ok=True)
    targets = {
        "32x32.png": 32,
        "128x128.png": 128,
        "128x128@2x.png": 256,
        "icon.png": 512,
    }
    for name, size in targets.items():
        path = os.path.join(ICONS, name)
        write_png(path, size)
        print(f"  {name} ({size}x{size})")
    print("Иконки готовы:", ICONS)


if __name__ == "__main__":
    main()
