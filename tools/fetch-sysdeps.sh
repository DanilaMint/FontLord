#!/usr/bin/env bash
# Скрипт загрузки системных зависимостей Tauri (webkit2gtk-4.1 и т.д.) в локальный
# префикс без root-доступа. Используется только в CI/локальной среде, где пакеты
# не установлены через пакетный менеджер.
#
#   tools/fetch-sysdeps.sh [target-dir]
#
# Итог: target-dir/sysroot/{usr,lib}/... и target-dir/sysroot/usr/lib/x86_64-linux-gnu/pkgconfig
# Итоговый env:
#   export PKG_CONFIG_PATH=<target>/sysroot/usr/lib/x86_64-linux-gnu/pkgconfig
#   export LD_LIBRARY_PATH=<target>/sysroot/usr/lib/x86_64-linux-gnu
set -euo pipefail

TARGET="${1:-.local-sys}"
SYSROOT="$TARGET/sysroot"
DEBS="$TARGET/debs"
mkdir -p "$SYSROOT" "$DEBS"

PACKAGES=(
  libwebkit2gtk-4.1-dev
  libjavascriptcoregtk-4.1-dev
  libsoup-3.0-dev
  libgtk-3-dev
  libglib2.0-dev
  libgstreamer1.0-dev
  libgstreamer-plugins-base1.0-dev
  libcairo2-dev
  libpango1.0-dev
  libssl-dev
  libxdo-dev
  libxkbcommon-dev
  libatk1.0-dev
  libharfbuzz-dev
  libfribidi-dev
  libepoxy-dev
  libwayland-dev
  libxrandr-dev
  libxi-dev
  libxcursor-dev
  libxinerama-dev
  libdbus-1-dev
  libsystemd-dev
)

echo "==> Resolving package URIs..."
URIS=$(apt-get install --print-uris --no-install-recommends -y \
  -o Dir::Cache::archives="$DEBS" "${PACKAGES[@]}" 2>/dev/null \
  | grep -oE "'http[^']+\.deb'" | tr -d "'")

if [ -z "$URIS" ]; then
  echo "!! Не удалось получить список пакетов (нужны apt-списки / сеть к archive.ubuntu.com)" >&2
  exit 1
fi

echo "==> Downloading $(echo "$URIS" | wc -l) packages..."
for uri in $URIS; do
  f="$DEBS/$(basename "$uri")"
  [ -f "$f" ] || curl -fsSL -o "$f" "$uri"
done

echo "==> Extracting into $SYSROOT..."
for deb in "$DEBS"/*.deb; do
  dpkg-deb -x "$deb" "$SYSROOT"
done

# Пакеты -dev содержат только символические ссылки на runtime-библиотеки.
# Если целевой файл уже установлен в системе, копируем его в sysroot,
# иначе линкер не найдёт библиотеку по ссылке.
echo "==> Resolving dangling symlinks..."
SYS_LIB="$SYSROOT/usr/lib/x86_64-linux-gnu"
SYS_LIBS="/usr/lib/x86_64-linux-gnu"
if [ -d "$SYS_LIB" ] && [ -d "$SYS_LIBS" ]; then
  while IFS= read -r -d '' link; do
    target=$(readlink "$link")
    case "$target" in
      /*) continue ;;
    esac
    if [ -e "$SYS_LIBS/$target" ]; then
      cp -L "$SYS_LIBS/$target" "$SYS_LIB/$target" 2>/dev/null || true
    fi
  done < <(find "$SYS_LIB" -maxdepth 1 -type l -print0)
fi

echo "==> Done. sysroot: $SYSROOT"
echo "    PKG_CONFIG_PATH=$SYSROOT/usr/lib/x86_64-linux-gnu/pkgconfig"
echo "    LD_LIBRARY_PATH=$SYSROOT/usr/lib/x86_64-linux-gnu:$SYSROOT/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1"
