PROFILE ?= dev

.PHONY: help run lint format test clean

help:
	@echo "使用可能なコマンド一覧:"
	@echo "  make run [PROFILE=dev|release] - アプリケーションを実行する（デフォルト: dev）"
	@echo "  make lint - コードを静的解析する"
	@echo "  make format - コードをフォーマットする"
	@echo "  make test - テストを実行する"
	@echo "  make clean - ビルド成果物を削除する"
	@echo "  make help - このヘルプメッセージを表示する"

run:
	@cargo run --bin dicom-server --profile ${PROFILE}

lint:
	@cargo clippy

format:
	@cargo fmt

test:
	@cargo test

clean:
	@cargo clean
