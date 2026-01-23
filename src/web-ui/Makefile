.PHONY: install run preview lint format test build help

.DEFAULT_GOAL := help

install:
	npm install

run:
	npm run dev

preview:
	npm run preview

lint:
	npm run lint

format:
	npm run format

test:
	npm run check

build:
	npm run build

help:
	@echo "web-ui Makefile"
	@echo ""
	@echo "使用方法: make [target]"
	@echo ""
	@echo "ターゲット:"
	@echo "  install  依存関係をインストール"
	@echo "  run      デバッグモードで起動"
	@echo "  preview  リリースモードで起動"
	@echo "  lint     リンターを実行"
	@echo "  format   フォーマッターを実行"
	@echo "  test     型チェックを実行"
	@echo "  build    リリースモードでビルド"
	@echo "  help     このヘルプを表示"
