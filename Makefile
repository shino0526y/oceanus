.PHONY: \
	up down logs preview _preview-run build psql test format clean help

.DEFAULT_GOAL := help

OS := $(shell uname -s)
CONTAINER_ENGINE ?= docker
COMPOSE := $(CONTAINER_ENGINE) compose
COMPOSE_PROD := $(COMPOSE) -f docker-compose.prod.yml

# 環境変数の読み込み
-include .env
export

# === 開発ワークフロー ===

up:
	$(COMPOSE) up -d

down:
	$(COMPOSE) down

logs:
	$(COMPOSE) logs -f

test:
	$(MAKE) -C src test

format:
	$(MAKE) -C src format

# === 本番準備・プレビュー ===

# ビルド引数
DB_USER ?= oceanus
DB_PASS ?= oceanus
DB_NAME ?= oceanus

build:
# 本番環境イメージをビルド
	$(CONTAINER_ENGINE) build --platform linux/amd64 -t oceanus-db:latest -f docker/db/Dockerfile .
	$(CONTAINER_ENGINE) build --platform linux/amd64 -t oceanus-web-ui:latest -f src/web-ui/Dockerfile src/web-ui
	$(CONTAINER_ENGINE) build --platform linux/amd64 --network=host --build-arg DATABASE_URL="postgres://$(DB_USER):$(DB_PASS)@localhost:5432/$(DB_NAME)" -t oceanus-dicom-server:latest --target dicom-server -f src/Dockerfile src
	$(CONTAINER_ENGINE) build --platform linux/amd64 --network=host --build-arg DATABASE_URL="postgres://$(DB_USER):$(DB_PASS)@localhost:5432/$(DB_NAME)" -t oceanus-web-api:latest --target web-api -f src/Dockerfile src
# 本番環境向けパッケージング
	rm -rf dist
	mkdir -p dist/docker/nginx dist/data/dicom
	$(COMPOSE_PROD) config --no-interpolate > dist/docker-compose.yml
# 生成された docker-compose.yml から build セクションを完全に削除し、絶対パスを相対パスに変換
	python3 -c "import re; p = 'dist/docker-compose.yml'; c = open(p).read(); c = re.sub(r' {4}build:[\s\S]+?(?=\n {4}\S)', '', c); open(p, 'w').write(c)"
ifeq ($(OS),Darwin)
	sed -i '' 's|$(PWD)|.|g' dist/docker-compose.yml
else
	sed -i 's|$(PWD)|.|g' dist/docker-compose.yml
endif
	cp .env.example dist/.env.example
	cp docker/nginx/default.conf dist/docker/nginx/default.conf
# イメージを保存
	IMAGES=$$($(COMPOSE_PROD) config | grep "image:" | awk '{print $$2}' | sort | uniq); \
	$(CONTAINER_ENGINE) save -o dist/oceanus-images.tar $$IMAGES
	@echo "完了: dist ディレクトリを確認してください。"

preview:
	@if [ ! -f .env ]; then \
		echo ".env が見つからないため .env.example から作成します..."; \
		cp .env.example .env; \
	fi
	@$(MAKE) _preview-run

_preview-run:
ifeq ($(OS),Darwin)
	@echo "macOS を検出しました。DICOM サーバーはローカルで(cargo run)、その他はコンテナで起動します。"
# dicom-server以外をコンテナでバックグラウンド起動
	$(COMPOSE_PROD) up -d db web-api web-ui
# dicom-serverはローカルで実行
	DATABASE_URL="postgres://oceanus:oceanus@localhost:5432/oceanus" \
	cargo run --manifest-path src/Cargo.toml --release -p dicom-server
# dicom-serverがCTRL+Cで停止したら、コンテナも停止
	$(COMPOSE_PROD) down
else
	$(COMPOSE_PROD) up
endif

# === メンテナンス ===

psql:
	$(COMPOSE) exec db psql -U $${POSTGRES_USER:-oceanus} -d $${POSTGRES_DB:-oceanus}

clean:
	$(COMPOSE) down -v --rmi local
	rm -rf dist src/target

# === ヘルプ ===

help:
	@echo "oceanus Makefile コマンド一覧"
	@echo ""
	@echo "開発:"
	@echo "  up      開発環境の起動"
	@echo "  down    開発環境の停止"
	@echo "  logs    ログの表示"
	@echo "  test    テストの実行"
	@echo "  format  コードの整形"
	@echo ""
	@echo "本番準備:"
	@echo "  build   配布用パッケージ(dist)の作成"
	@echo "  preview 本番構成での動作確認"
	@echo ""
	@echo "メンテナンス:"
	@echo "  psql    データベースへの接続"
	@echo "  clean   環境の完全初期化"
	@echo ""
	@echo "  help    このヘルプを表示"
