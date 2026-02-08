.PHONY: \
	up down start stop restart logs ps build preview pull clean \
	prod-up prod-down prod-start prod-stop prod-restart prod-logs prod-ps prod-build \
	db-up db-start db-stop db-restart db-logs db-shell db-psql \
	install lint format test src-build src-clean \
	help

.DEFAULT_GOAL := help

COMPOSE := docker compose

# === Docker Compose 操作 ===

up:
	$(COMPOSE) up -d

down:
	$(COMPOSE) down

start:
	$(COMPOSE) start

stop:
	$(COMPOSE) stop

restart:
	$(COMPOSE) restart

logs:
	$(COMPOSE) logs -f

ps:
	$(COMPOSE) ps -a

preview:
	@echo "本番環境イメージのビルドおよびプレビュー起動..."
	@echo "停止するには Ctrl+C を押してください。"
	$(COMPOSE) -f docker-compose.prod.yml build
	$(COMPOSE) -f docker-compose.prod.yml up
	@echo "プレビューを停止しました。"

build:
	@echo "本番環境イメージをビルド中..."
	$(COMPOSE) -f docker-compose.prod.yml build
	@echo "本番環境向けパッケージング中..."
	rm -rf dist
	mkdir -p dist/docker/nginx
	mkdir -p dist/data/dicom
	# dist 用の docker-compose.yml を生成 (オフラインデプロイ用にビルド関連の行を削除)
	cat docker-compose.prod.yml | grep -vE "build:|context:|dockerfile:|target:" > dist/docker-compose.yml
	cp .env.example dist/.env.example
	cp docker/nginx/default.conf dist/docker/nginx/default.conf
	@echo "イメージを tarball に保存中 (時間がかかる場合があります)..."
	# docker-compose.prod.yml からイメージ名を取得して保存
	IMAGES=$$($(COMPOSE) -f docker-compose.prod.yml config | grep "image:" | awk '{print $$2}' | sort | uniq); \
	docker save -o dist/oceanus-images.tar $$IMAGES
	@echo "完了。本番環境ファイルは 'dist' ディレクトリにあります。"
	@echo "デプロイ方法: 'dist' をサーバーにコピーし、'docker load -i oceanus-images.tar' および 'docker compose up -d' を実行してください。"

pull:
	$(COMPOSE) pull

clean:
	$(COMPOSE) down -v --rmi local

# === Production (Full Stack) 操作 ===

prod-up:
	$(COMPOSE) -f docker-compose.prod.yml up -d

prod-down:
	$(COMPOSE) -f docker-compose.prod.yml down

prod-start:
	$(COMPOSE) -f docker-compose.prod.yml start

prod-stop:
	$(COMPOSE) -f docker-compose.prod.yml stop

prod-restart:
	$(COMPOSE) -f docker-compose.prod.yml restart

prod-logs:
	$(COMPOSE) -f docker-compose.prod.yml logs -f

prod-ps:
	$(COMPOSE) -f docker-compose.prod.yml ps -a

prod-build:
	$(COMPOSE) -f docker-compose.prod.yml build

# === DB 個別操作 ===

db-up:
	$(COMPOSE) up -d db

db-start:
	$(COMPOSE) start db

db-stop:
	$(COMPOSE) stop db

db-restart:
	$(COMPOSE) restart db

db-logs:
	$(COMPOSE) logs -f db

db-shell:
	$(COMPOSE) exec db bash

db-psql:
	$(COMPOSE) exec db psql -U $${POSTGRES_USER:-oceanus} -d $${POSTGRES_DB:-oceanus}

# === src 配下のプロジェクト操作 ===

install:
	$(MAKE) -C src install

lint:
	$(MAKE) -C src lint

format:
	$(MAKE) -C src format

test:
	$(MAKE) -C src test

src-build:
	$(MAKE) -C src build

src-clean:
	$(MAKE) -C src clean

# === ヘルプ ===

help:
	@echo "oceanus Makefile"
	@echo ""
	@echo "使用方法: make [target]"
	@echo ""
	@echo "Docker Compose:"
	@echo "  up        コンテナをバックグラウンドで起動"
	@echo "  down      コンテナを停止・削除"
	@echo "  start     停止中のコンテナを起動"
	@echo "  stop      コンテナを停止"
	@echo "  restart   コンテナを再起動"
	@echo "  logs      ログを表示 (follow)"
	@echo "  ps        コンテナ一覧を表示"
	@echo "  build     イメージをビルド"
	@echo "  preview   本番環境イメージをビルドしてプレビュー起動 (Ctrl+Cで停止)"
	@echo "  pull      イメージをプル"
	@echo "  clean     コンテナ・ボリューム・イメージを削除"
	@echo ""
	@echo "Production (Full Stack):"
	@echo "  prod-up      本番環境構成を起動"
	@echo "  prod-down    本番環境構成を停止・削除"
	@echo "  prod-logs    本番環境のログを表示"
	@echo ""
	@echo "DB 操作:"
	@echo "  db-up      DBコンテナを起動"
	@echo "  db-start   DBコンテナを起動"
	@echo "  db-stop    DBコンテナを停止"
	@echo "  db-restart DBコンテナを再起動"
	@echo "  db-logs    DBコンテナのログを表示"
	@echo "  db-shell   DBコンテナにシェルで接続"
	@echo "  db-psql    DBコンテナにpsqlで接続"
	@echo ""
	@echo "ソースコード (src/):"
	@echo "  install    全プロジェクトの依存関係をインストール"
	@echo "  lint       全プロジェクトでリンターを実行"
	@echo "  format     全プロジェクトでフォーマッターを実行"
	@echo "  test       全プロジェクトでテストを実行"
	@echo "  src-build  全プロジェクトをビルド"
	@echo "  src-clean  全プロジェクトのビルド成果物を削除"
	@echo ""
	@echo "  help       このヘルプを表示"
