PROFILE ?= dev
MAKEFLAGS += --no-print-directory
EXTRA := $(filter-out help run run-dicom-server lint format test clean, $(MAKECMDGOALS))

.PHONY: help run run-dicom-server lint format test clean

help:
	@echo "使用可能なコマンド一覧:"
	@echo "  make run <AE-TITLE> [PROFILE=dev|release] - アプリケーションを実行する（AEタイトルは必須、プロファイルは省略可（デフォルト: dev））"
	@echo "    例: make run ANY-SCP PROFILE=release"
	@echo "  make run-dicom-server <AE-TITLE> [PROFILE=dev|release] - DICOMサーバーを単体で実行する（AEタイトルは必須、プロファイルは省略可（デフォルト: dev））"
	@echo "    例: make run-dicom-server ANY-SCP PROFILE=release"
	@echo "  make lint - コードを静的解析する"
	@echo "  make format - コードをフォーマットする"
	@echo "  make test - テストを実行する"
	@echo "  make clean - ビルド成果物を削除する"
	@echo "  make help - このヘルプメッセージを表示する"

run:
	@make run-dicom-server PROFILE=$(PROFILE) -- $(EXTRA)

run-dicom-server:
	@cd src/dicom-server && make run PROFILE=$(PROFILE) -- $(EXTRA)

lint:
	@cd src && cargo clippy

format:
	@cd src && cargo fmt

test:
	@cd src && cargo test

clean:
	@cd src && cargo clean
