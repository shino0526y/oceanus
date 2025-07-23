PROFILE ?= dev

.PHONY: help init run run-dicom-server lint format test exit

help:
	@echo "使用可能なコマンド一覧:"
	@echo "  make init - 環境を初期化する"
	@echo "  make run [PROFILE=dev|release] - アプリケーションを実行する（デフォルト: dev）"
	@echo "  make run-dicom-server [PROFILE=dev|release] - DICOMサーバーを単体で実行する（デフォルト: dev）"
	@echo "  make lint - コードを静的解析する"
	@echo "  make format - コードをフォーマットする"
	@echo "  make test - テストを実行する"
	@echo "  make exit - 環境を停止する"
	@echo "  make help - このヘルプメッセージを表示する"

init:
	@docker compose up -d

run:
	@make run-dicom-server PROFILE=${PROFILE}

run-dicom-server:
	@sh -c "cd src/dicom-server && make run PROFILE=${PROFILE}"

lint:
	@docker exec -it rust sh -c "cd src && cargo clippy"

format:
	@docker exec -it rust sh -c "cd src && cargo fmt"

test:
	@docker exec -it rust sh -c "cd src && cargo test"

exit:
	@docker compose down
