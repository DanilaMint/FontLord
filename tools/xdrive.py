#!/usr/bin/env python3
"""Управление указателем через XTEST (для автоматической проверки GUI).

Пример:

    python3 tools/xdrive.py click 400 300
    python3 tools/xdrive.py drag 400 600 500 300
"""

from __future__ import annotations

import ctypes
import ctypes.util
import sys
import time

x11 = ctypes.CDLL(ctypes.util.find_library("X11"))
xtst = ctypes.CDLL(ctypes.util.find_library("Xtst"))

x11.XOpenDisplay.restype = ctypes.c_void_p
x11.XOpenDisplay.argtypes = [ctypes.c_char_p]
x11.XFlush.argtypes = [ctypes.c_void_p]
xtst.XTestFakeMotionEvent.argtypes = [
    ctypes.c_void_p,
    ctypes.c_int,
    ctypes.c_int,
    ctypes.c_int,
    ctypes.c_ulong,
]
xtst.XTestFakeButtonEvent.argtypes = [
    ctypes.c_void_p,
    ctypes.c_uint,
    ctypes.c_bool,
    ctypes.c_ulong,
]


class Display:
    """Обёртка над X11-дисплеем."""

    def __init__(self, name: str | None = None) -> None:
        self.handle = x11.XOpenDisplay(name.encode() if name else None)
        if not self.handle:
            raise RuntimeError("не удалось открыть дисплей")

    def move(self, x: int, y: int) -> None:
        xtst.XTestFakeMotionEvent(self.handle, -1, x, y, 0)
        x11.XFlush(self.handle)

    def button(self, button: int, pressed: bool) -> None:
        xtst.XTestFakeButtonEvent(self.handle, button, pressed, 0)
        x11.XFlush(self.handle)

    def click(self, x: int, y: int, button: int = 1, delay: float = 0.05) -> None:
        self.move(x, y)
        time.sleep(delay)
        self.button(button, True)
        time.sleep(delay)
        self.button(button, False)
        time.sleep(delay)

    def drag(self, points: list[tuple[int, int]], button: int = 1, steps: int = 12) -> None:
        """Рисование ломаной с равномерным распределением шагов."""
        self.move(*points[0])
        self.button(button, True)
        time.sleep(0.05)
        for start, end in zip(points, points[1:]):
            for step in range(1, steps + 1):
                t = step / steps
                self.move(
                    int(start[0] + (end[0] - start[0]) * t),
                    int(start[1] + (end[1] - start[1]) * t),
                )
                time.sleep(0.012)
        self.button(button, False)
        time.sleep(0.1)


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__, file=sys.stderr)
        return 2

    display = Display()
    command = argv[1]

    if command == "click":
        display.click(int(argv[2]), int(argv[3]))
    elif command == "drag":
        coordinates = [int(value) for value in argv[2:]]
        points = list(zip(coordinates[::2], coordinates[1::2]))
        display.drag(points)
    else:
        print(f"неизвестная команда: {command}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
