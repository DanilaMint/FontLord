#!/usr/bin/env bash
# Переменные окружения для сборки Fontlord в окружениях без root-доступа.
#
# Скрипт `tools/fetch-sysdeps.sh` раскладывает системные зависимости Tauri
# (webkit2gtk-4.1, gtk3 и т.д.) в локальный sysroot. Этот файл подключает
# его к сборке:
#
#   source tools/env.sh
#   cargo build
#
SHELL_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SYSROOT="$SHELL_ROOT/.local-sys/sysroot"
LIBDIR="$SYSROOT/usr/lib/x86_64-linux-gnu"

export CARGO_HOME="${CARGO_HOME:-$SHELL_ROOT/.cargo-home}"
export PKG_CONFIG_PATH="$LIBDIR/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
export LD_LIBRARY_PATH="$LIBDIR:$LIBDIR/webkit2gtk-4.1${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export RUSTFLAGS="-L native=$LIBDIR -L native=$LIBDIR/webkit2gtk-4.1${RUSTFLAGS:+ $RUSTFLAGS}"
export FONTLORD_SYSROOT="$SYSROOT"
