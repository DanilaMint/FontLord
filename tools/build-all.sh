#!/usr/bin/env bash
# Сборка Fontlord под все доступные платформы.
#
# Скрипт выполняет три шага:
#
#   1. проверяет компиляцию ядра (fontlord-core, fontlord-ttf, fontlord-cli)
#      для macOS, Windows и Linux arm64 — без системных SDK сборка не
#      требуется;
#   2. собирает исполняемые файлы консольных утилит для всех целей, для
#      которых есть линкер (Windows — через mingw-w64);
#   3. собирает установочные пакеты приложения для текущей платформы
#      (deb, rpm, AppImage на Linux).
#
# Графическое приложение для macOS и Windows собирается только на
# нативных раннерах (GitHub Actions), поскольку требует системных SDK:
# webkit2gtk для Linux, WebView2 для Windows, AppKit для macOS.
#
# Использование:
#
#   ./tools/build-all.sh            # всё
#   ./tools/build-all.sh --check    # только проверка компиляции ядра
#
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# shellcheck source=./env.sh
source tools/env.sh

CARGO="${CARGO:-cargo}"
MINGW_DIR="${MINGW_DIR:-$ROOT/.probe/mingw/root/usr/bin}"
TAURI="$ROOT/ui/node_modules/.bin/tauri"

# Цели, для которых проверяется компиляция ядра.
CHECK_TARGETS=(
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
  aarch64-apple-darwin
  x86_64-apple-darwin
  x86_64-pc-windows-msvc
  aarch64-pc-windows-msvc
)

# Цели, для которых собираются готовые исполняемые файлы CLI.
BUILD_TARGETS=(
  x86_64-unknown-linux-gnu
  x86_64-pc-windows-gnu
)

ONLY_CHECK=0
[[ "${1:-}" == "--check" ]] && ONLY_CHECK=1

log() {
  printf '\n\033[1;36m==> %s\033[0m\n' "$1"
}

installed_targets() {
  rustup target list --installed 2>/dev/null || true
}

# 1. Проверка компиляции ядра для всех целей.
log "Проверка компиляции ядра под разные платформы"
INSTALLED="$(installed_targets)"
for target in "${CHECK_TARGETS[@]}"; do
  if ! grep -qx "$target" <<<"$INSTALLED"; then
    printf '  %-28s таргет не установлен (rustup target add %s)\n' "$target" "$target"
    continue
  fi
  if $CARGO check --quiet --target "$target" \
      -p fontlord-core -p fontlord-ttf -p fontlord-cli 2>/dev/null; then
    printf '  %-28s ok\n' "$target"
  else
    printf '  %-28s ошибка компиляции\n' "$target"
  fi
done

# 2. Сборка исполняемых файлов консольных утилит.
if [[ $ONLY_CHECK -eq 0 ]]; then
  log "Сборка консольных утилит (fontlord-cli)"
  for target in "${BUILD_TARGETS[@]}"; do
    if ! grep -qx "$target" <<<"$INSTALLED"; then
      printf '  %-28s таргет не установлен\n' "$target"
      continue
    fi
    if [[ "$target" == *windows-gnu ]]; then
      linker="$MINGW_DIR/x86_64-w64-mingw32-gcc-posix"
      if [[ ! -x "$linker" ]]; then
        printf '  %-28s нет mingw-w64 (нужен линкер)\n' "$target"
        continue
      fi
      export PATH="$MINGW_DIR:$PATH"
      if env "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=$linker" \
          $CARGO build --quiet --release --target "$target" -p fontlord-cli \
          >/dev/null 2>&1; then
        printf '  %-28s ok\n' "$target"
      else
        printf '  %-28s ошибка сборки\n' "$target"
      fi
      continue
    fi
    if $CARGO build --quiet --release --target "$target" -p fontlord-cli >/dev/null 2>&1; then
      printf '  %-28s ok\n' "$target"
    else
      printf '  %-28s ошибка сборки\n' "$target"
    fi
  done
fi

# 3. Установочные пакеты приложения для текущей платформы.
if [[ $ONLY_CHECK -eq 0 && -x "$TAURI" ]]; then
  log "Сборка установочных пакетов приложения (текущая платформа)"
  case "$(uname -s)" in
    Linux)   "$TAURI" build --bundles deb,appimage,rpm ;;
    Darwin)  "$TAURI" build --bundles app,dmg ;;
    MINGW*|MSYS*|CYGWIN*) "$TAURI" build --bundles msi,nsis ;;
    *)       "$TAURI" build ;;
  esac
fi

log "Готово"
find target/release/bundle -maxdepth 2 -type f \( -name '*.deb' -o -name '*.rpm' \
  -o -name '*.AppImage' -o -name '*.dmg' -o -name '*.msi' -o -name '*.exe' \) \
  -printf '  %p (%s байт)\n' 2>/dev/null || true
