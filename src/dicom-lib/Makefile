PROFILE ?= dev

.PHONY: help lint format test clean

help:
	@echo "使用可能なコマンド一覧:"
	@echo "  make lint - コードを静的解析する"
	@echo "  make format - コードをフォーマットする"
	@echo "  make test - テストを実行する"
	@echo "  make clean - ビルド成果物を削除する"
	@echo "  make help - このヘルプメッセージを表示する"

lint:
	@cargo clippy

format:
	@cargo fmt

test:
	@cargo test

clean:
	@cargo clean
