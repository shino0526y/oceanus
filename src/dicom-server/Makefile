PROFILE ?= dev
EXTRA := $(filter-out help run lint format test clean, $(MAKECMDGOALS))

.PHONY: help run lint format test clean

help:
	@echo "使用可能なコマンド一覧:"
	@echo "  make run <AE-TITLE> [PROFILE=dev|release] - アプリケーションを実行する（AEタイトルは必須、プロファイルは省略可（デフォルト: dev））"
	@echo "    例: make run ANY-SCP PROFILE=release"
	@echo "  make lint - コードを静的解析する"
	@echo "  make format - コードをフォーマットする"
	@echo "  make test - テストを実行する"
	@echo "  make clean - ビルド成果物を削除する"
	@echo "  make help - このヘルプメッセージを表示する"

run:
	@cargo run --bin dicom-server --profile $(PROFILE) -- $(EXTRA)

lint:
	@cargo clippy

format:
	@cargo fmt

test:
	@cargo test

clean:
	@cargo clean
