CONTAINER_ENGINE ?= docker
COMPOSE          := $(CONTAINER_ENGINE) compose
COMPOSE_PROD     := $(COMPOSE) -f docker-compose.prod.yml
OS               := $(shell uname -s)

DB_USER      ?= oceanus
DB_PASS      ?= oceanus
DB_NAME      ?= oceanus
DATABASE_URL ?= postgres://$(DB_USER):$(DB_PASS)@localhost:5432/$(DB_NAME)

ARCH         ?= $(shell uname -m | sed -e 's/x86_64/amd64/' -e 's/arm64/arm64/' -e 's/aarch64/arm64/')
PLATFORM     ?= linux/$(ARCH)
DIST_DIR     := dist
IMG_PREFIX   := oceanus
IMG_DB           := $(IMG_PREFIX)-db:latest
IMG_WEB_UI       := $(IMG_PREFIX)-web-ui:latest
IMG_DICOM_SERVER := $(IMG_PREFIX)-dicom-server:latest
IMG_WEB_API      := $(IMG_PREFIX)-web-api:latest

-include .env
export

.PHONY: help
help: # このヘルプを表示
	@echo "oceanus Makefile コマンド一覧"
	@echo ""
	@echo "開発:"
	@echo "  up             開発環境の起動"
	@echo "  down           開発環境の停止"
	@echo "  logs           ログの表示"
	@echo "  test           テストの実行"
	@echo "  format         コードの整形"
	@echo ""
	@echo "本番準備:"
	@echo "  build          配布用パッケージ(dist)の作成 (オプション: ARCH=amd64|arm64)"
	@echo "  preview        本番構成での動作確認"
	@echo ""
	@echo "メンテナンス:"
	@echo "  psql           データベース接続"
	@echo "  sqlx-prepare   SQLx メタデータの生成"
	@echo "  clean          環境のクリーンアップ"

.DEFAULT_GOAL := help

.PHONY: up
up: # 開発環境の起動
	$(COMPOSE) up -d

.PHONY: down
down: # 開発環境の停止
	$(COMPOSE) down

.PHONY: logs
logs: # ログの表示
	$(COMPOSE) logs -f

.PHONY: test
test: # テストの実行
	$(MAKE) -C src test

.PHONY: format
format: # コードの整形
	$(MAKE) -C src format

.PHONY: sqlx-prepare
sqlx-prepare: # SQLxメタデータの生成
	cd src && DATABASE_URL="$(DATABASE_URL)" cargo sqlx prepare --workspace -- --all-targets --all-features

.PHONY: build
build: # 配布用パッケージ(dist)の作成
	$(CONTAINER_ENGINE) build --platform $(PLATFORM) -t $(IMG_DB) -f docker/db/Dockerfile .
	$(CONTAINER_ENGINE) build --platform $(PLATFORM) -t $(IMG_WEB_UI) -f src/web-ui/Dockerfile src/web-ui
	$(CONTAINER_ENGINE) build --platform $(PLATFORM) -t $(IMG_DICOM_SERVER) --target dicom-server -f src/Dockerfile src
	$(CONTAINER_ENGINE) build --platform $(PLATFORM) -t $(IMG_WEB_API) --target web-api -f src/Dockerfile src
	rm -rf $(DIST_DIR)
	mkdir -p $(DIST_DIR)/docker/nginx
	awk '/^    build:/{skip=1;next} skip && /^    [^ ]/{skip=0} skip{next} {print}' docker-compose.prod.yml > $(DIST_DIR)/docker-compose.yml
	cp .env.example $(DIST_DIR)/.env.example
	cp docker/nginx/default.conf $(DIST_DIR)/docker/nginx/default.conf
	@IMAGES=$$($(COMPOSE_PROD) config | grep "image:" | awk '{print $$2}' | sort | uniq); \
	$(CONTAINER_ENGINE) save -o $(DIST_DIR)/oceanus-images.tar $$IMAGES

.PHONY: preview
preview: # 本番構成での動作確認
	@if [ ! -f .env ]; then cp .env.example .env; fi
ifeq ($(OS),Darwin)
# dicom-serverがDICOM通信を行うにあたり、通信先のホスト名(IPアドレス)とポートが事前に登録されているものと一致している必要があるため、
# dicom-serverコンテナはホストネットワークを使用する構成となっている。
# しかしながら、macOSのDocker Desktopはホストネットワークをサポートしていない。
# そのため、dicom-serverについてはコンテナではなくローカルで直接実行する構成とする。

# dicom-server以外のコンテナを起動し、終了時に必ず停止するようにtrapで設定する。
# dicom-serverについてはローカルで直接実行する。
# なお、dicom-serverはlocalhostの5432ポートでDBに接続する必要があるため、DATABASE_URLのホスト名を@db:からlocalhost:に置換し、起動する。
	$(COMPOSE_PROD) up -d db web-api web-ui
	trap '$(COMPOSE_PROD) down' EXIT; \
	DATABASE_URL="$(subst @db:,@localhost:,$(DATABASE_URL))" \
	cargo run --manifest-path src/Cargo.toml --release -p dicom-server
else
	$(COMPOSE_PROD) up
	$(COMPOSE_PROD) down
endif

.PHONY: psql
psql: # データベース接続
	$(COMPOSE) exec db psql -U $(DB_USER) -d $(DB_NAME)

.PHONY: clean
clean: # 環境のクリーンアップ
	$(COMPOSE) down -v --rmi local
	rm -rf $(DIST_DIR) src/target
