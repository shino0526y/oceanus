PROFILE ?= dev

.PHONY: help run lint format test

help:
	@echo "使用可能なコマンド一覧:"
	@echo "  make run [PROFILE=dev|release] - アプリケーションを実行する（デフォルト: dev）"
	@echo "  make lint - コードを静的解析する"
	@echo "  make format - コードをフォーマットする"
	@echo "  make test - テストを実行する"
	@echo "  make help - このヘルプメッセージを表示する"

run:
	@docker exec -it rust sh -c "cd src/dicom-server && cargo run --bin dicom-server --profile ${PROFILE}"

lint:
	@docker exec -it rust sh -c "cd src/dicom-server && cargo clippy"

format:
	@docker exec -it rust sh -c "cd src/dicom-server && cargo fmt"

test:
	@docker exec -it rust sh -c "cd src/dicom-server && cargo test"
