.PHONY: install run preview lint format test build clean help

.DEFAULT_GOAL := help

install:
	cargo fetch

run:
	@echo "dicom-lib はライブラリのため、単体では実行できません。"
	@echo "dicom-server または web-api から使用してください。"

preview:
	@echo "dicom-lib はライブラリのため、単体では実行できません。"
	@echo "dicom-server または web-api から使用してください。"

lint:
	cargo clippy -- -D warnings

format:
	cargo fmt

test:
	cargo test

build:
	cargo build --release

clean:
	cargo clean

help:
	@echo "dicom-lib Makefile"
	@echo ""
	@echo "使用方法: make [target]"
	@echo ""
	@echo "ターゲット:"
	@echo "  install  依存関係をインストール"
	@echo "  run      デバッグモードで起動（ライブラリのため実行不可）"
	@echo "  preview  リリースモードで起動（ライブラリのため実行不可）"
	@echo "  lint     リンターを実行"
	@echo "  format   フォーマッターを実行"
	@echo "  test     テストを実行"
	@echo "  build    リリースモードでビルド"
	@echo "  clean    ビルド成果物を削除"
	@echo "  help     このヘルプを表示"
