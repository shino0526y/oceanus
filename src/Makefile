.PHONY: install run preview lint format test build help \
        install-all run-all preview-all lint-all format-all test-all build-all

.DEFAULT_GOAL := help

PROJECTS := dicom-lib dicom-server web-api web-ui

# 全プロジェクトに対して実行
install-all:
	@for dir in $(PROJECTS); do \
		echo "=== $$dir: install ==="; \
		$(MAKE) -C $$dir install; \
	done

run-all:
	@echo "run-all は各プロジェクトを個別に起動してください。"
	@echo "  make run-dicom-server"
	@echo "  make run-web-api"
	@echo "  make run-web-ui"

preview-all:
	@echo "preview-all は各プロジェクトを個別に起動してください。"
	@echo "  make preview-dicom-server"
	@echo "  make preview-web-api"
	@echo "  make preview-web-ui"

lint-all:
	@for dir in $(PROJECTS); do \
		echo "=== $$dir: lint ==="; \
		$(MAKE) -C $$dir lint; \
	done

format-all:
	@for dir in $(PROJECTS); do \
		echo "=== $$dir: format ==="; \
		$(MAKE) -C $$dir format; \
	done

test-all:
	@for dir in $(PROJECTS); do \
		echo "=== $$dir: test ==="; \
		$(MAKE) -C $$dir test; \
	done

build-all:
	@for dir in $(PROJECTS); do \
		echo "=== $$dir: build ==="; \
		$(MAKE) -C $$dir build; \
	done

# 個別プロジェクトに対して実行
install-%:
	$(MAKE) -C $* install

run-%:
	$(MAKE) -C $* run

preview-%:
	$(MAKE) -C $* preview

lint-%:
	$(MAKE) -C $* lint

format-%:
	$(MAKE) -C $* format

test-%:
	$(MAKE) -C $* test

build-%:
	$(MAKE) -C $* build

# エイリアス
install: install-all
lint: lint-all
format: format-all
test: test-all
build: build-all

help:
	@echo "oceanus Makefile"
	@echo ""
	@echo "使用方法: make [target]"
	@echo ""
	@echo "全プロジェクト対象:"
	@echo "  install      全プロジェクトの依存関係をインストール"
	@echo "  lint         全プロジェクトでリンターを実行"
	@echo "  format       全プロジェクトでフォーマッターを実行"
	@echo "  test         全プロジェクトでテストを実行"
	@echo "  build        全プロジェクトをリリースモードでビルド"
	@echo ""
	@echo "個別プロジェクト対象 (例: make run-web-api):"
	@echo "  install-<project>  依存関係をインストール"
	@echo "  run-<project>      デバッグモードで起動"
	@echo "  preview-<project>  リリースモードで起動"
	@echo "  lint-<project>     リンターを実行"
	@echo "  format-<project>   フォーマッターを実行"
	@echo "  test-<project>     テストを実行"
	@echo "  build-<project>    リリースモードでビルド"
	@echo ""
	@echo "プロジェクト一覧:"
	@echo "  dicom-lib, dicom-server, web-api, web-ui"
	@echo ""
	@echo "  help         このヘルプを表示"
