# Fontlord — команды разработки.
#
#   make deps     установить системные зависимости Tauri в локальный sysroot
#   make env      подключить sysroot к окружению (source tools/env.sh)
#   make deps-ui  установить зависимости фронтенда
#   make dev      запустить приложение в режиме разработки
#   make build    собрать релизную версию
#   make test     прогнать тесты
#   make check    форматирование, статический анализ и тесты
#   make icons    перегенерировать иконки приложения
#   make dist     собрать под все доступные платформы (tools/build-all.sh)
#   make clean    удалить артефакты сборки

CARGO ?= cargo
PNPM ?= pnpm

.PHONY: help deps env deps-ui dev build bundle dist test check fmt clippy icons clean

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}'

deps: ## Установить системные зависимости Tauri без root-доступа
	./tools/fetch-sysdeps.sh .local-sys

env: ## Подключить локальный sysroot (нужно вызвать как source)
	@echo "source tools/env.sh"

deps-ui: ## Установить зависимости фронтенда
	cd ui && $(PNPM) install

dev: ## Запустить приложение в режиме разработки
	./ui/node_modules/.bin/tauri dev

build: ## Собрать релизную версию приложения
	cd ui && $(PNPM) build
	$(CARGO) build --release -p fontlord -p fontlord-cli

bundle: ## Собрать установочные пакеты (deb, rpm, AppImage, dmg, msi, nsis)
	./ui/node_modules/.bin/tauri build

dist: ## Собрать под все доступные платформы
	./tools/build-all.sh

test: ## Прогнать тесты
	$(CARGO) test --workspace

fmt: ## Отформатировать код
	$(CARGO) fmt --all

clippy: ## Статический анализ
	$(CARGO) clippy --workspace --all-targets -- -D warnings

check: fmt clippy test ## Форматирование, анализ и тесты

icons: ## Перегенерировать иконки приложения
	$(PNPM) --dir ui tauri icon src-tauri/icons/icon.png

clean: ## Удалить артефакты сборки
	$(CARGO) clean
	rm -rf ui/dist
