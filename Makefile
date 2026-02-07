.PHONY: \
	up down start stop restart logs ps build pull clean \
	db-up db-down db-start db-stop db-restart db-logs db-shell db-psql \
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

build:
	$(COMPOSE) build

pull:
	$(COMPOSE) pull

clean:
	$(COMPOSE) down -v --rmi local

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
	@echo "  pull      イメージをプル"
	@echo "  clean     コンテナ・ボリューム・イメージを削除"
	@echo ""
	@echo "DB 操作:"
	@echo "  db-up      DBコンテナを起動"
	@echo "  db-down    DBコンテナを停止・削除"
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
