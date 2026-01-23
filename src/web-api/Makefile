.PHONY: install run preview lint format test build help

.DEFAULT_GOAL := help

install:
	cargo fetch

run:
	cargo run

preview:
	cargo run --release

lint:
	cargo clippy -- -D warnings

format:
	cargo fmt

test:
	cargo test

build:
	cargo build --release

help:
	@echo "web-api Makefile"
	@echo ""
	@echo "使用方法: make [target]"
	@echo ""
	@echo "ターゲット:"
	@echo "  install  依存関係をインストール"
	@echo "  run      デバッグモードで起動"
	@echo "  preview  リリースモードで起動"
	@echo "  lint     リンターを実行"
	@echo "  format   フォーマッターを実行"
	@echo "  test     テストを実行"
	@echo "  build    リリースモードでビルド"
	@echo "  help     このヘルプを表示"
