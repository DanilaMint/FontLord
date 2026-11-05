#!/usr/bin/env python3
"""Конвертация снимка экрана X (формат XWD) в PNG.

Используется для автоматической проверки приложения в headless-окружении:

    xwd -root -silent > screen.xwd
    python3 tools/xwd2png.py screen.xwd screen.png
"""

from __future__ import annotations

import struct
import sys

from PIL import Image


def convert(source: str, target: str) -> None:
    data = open(source, "rb").read()
    header = struct.unpack(">25I", data[:100])
    header_size = header[0]
    width = header[4]
    height = header[5]
    bytes_per_line = header[12]
    colors = header[18]

    offset = header_size + colors * 12
    pixels = data[offset : offset + bytes_per_line * height]

    image = Image.new("RGB", (width, height))
    target_pixels = image.load()
    for y in range(height):
        row = pixels[y * bytes_per_line : (y + 1) * bytes_per_line]
        for x in range(width):
            base = x * 4
            target_pixels[x, y] = (row[base + 2], row[base + 1], row[base])
    image.save(target)
    print(f"{target}: {width}x{height}")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("usage: xwd2png.py <input.xwd> <output.png>", file=sys.stderr)
        raise SystemExit(2)
    convert(sys.argv[1], sys.argv[2])
