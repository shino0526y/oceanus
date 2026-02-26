# Oceanus

Rustで実装されたPACSです。

## 構成

| コンポーネント   | 説明                                   |
| ---------------- | -------------------------------------- |
| **dicom-server** | DICOMサーバー                          |
| **web-api**      | REST API サーバー (Axum)               |
| **web-ui**       | Web フロントエンド (SvelteKit + nginx) |
| **db**           | PostgreSQL データベース                |

## 前提条件

- Docker
- Rust ツールチェーン
- Volta

## 開発

### セットアップ

```bash
cp .env.example .env  # 必要に応じて値を編集
```

### コマンド一覧

```
make help
```

| コマンド            | 説明                                            |
| ------------------- | ----------------------------------------------- |
| `make up`           | 開発環境の起動 (DBのみ)                         |
| `make down`         | 開発環境の停止                                  |
| `make logs`         | ログの表示                                      |
| `make test`         | テストの実行                                    |
| `make format`       | コードの整形                                    |
| `make psql`         | データベースに接続                              |
| `make sqlx-prepare` | SQLx メタデータの生成                           |
| `make clean`        | 環境のクリーンアップ (ボリューム・イメージ含む) |

## 本番デプロイ

### 配布用パッケージの作成

```bash
make build                # ホストと同じアーキテクチャでビルド
make build ARCH=amd64     # amd64 を指定してビルド
```

`dist/` ディレクトリに以下が生成されます:

```
dist/
├── .env.example
├── docker-compose.yml
├── docker/
│   └── nginx/
│       ├── default.conf
│       └── security_headers.conf
└── oceanus-images.tar
```

### デプロイ先での起動

```bash
# dist/ をデプロイ先に配置
cp .env.example .env      # 環境変数を環境に合わせて編集
docker load -i oceanus-images.tar
docker compose up -d
```

### 本番構成のローカルプレビュー

```bash
make preview
```

> **macOS の場合**: Docker Desktop はホストネットワークをサポートしていないため、`dicom-server` はコンテナではなくローカルで直接実行されます。

### 環境変数

| 変数名                      | 説明                                | デフォルト値                                        |
| --------------------------- | ----------------------------------- | --------------------------------------------------- |
| `POSTGRES_DB`               | データベース名                      | `oceanus`                                           |
| `POSTGRES_USER`             | データベースユーザー                | `oceanus`                                           |
| `POSTGRES_PASSWORD`         | データベースパスワード              | `oceanus`                                           |
| `AE_TITLE`                  | DICOM AE タイトル                   | `OCEANUS`                                           |
| `DICOM_PORT`                | DICOM サーバーポート                | `104`                                               |
| `DATA_DIR`                  | データディレクトリ                  | `/var/lib/oceanus`                                  |
| `DATABASE_URL`              | データベース接続 URL                | `postgres://oceanus:oceanus@db:5432/oceanus`        |
| `DICOM_SERVER_DATABASE_URL` | dicom-server 用データベース接続 URL | `postgres://oceanus:oceanus@localhost:5432/oceanus` |

> `dicom-server` は `network_mode: host` で動作するため、DB への接続先が `localhost` になります。

## DICOM Server

### 対応するサービス

- Verification
  - Verification SOP Class (1.2.840.10008.1.1)
- Storage
  - Computed Radiography Image Storage (1.2.840.10008.5.1.4.1.1.1)
  - Digital X-Ray Image Storage - For Presentation (1.2.840.10008.5.1.4.1.1.1.1)
  - Digital Mammography X-Ray Image Storage - For Presentation (1.2.840.10008.5.1.4.1.1.1.2)
  - CT Image Storage (1.2.840.10008.5.1.4.1.1.2)
  - MR Image Storage (1.2.840.10008.5.1.4.1.1.4.2)
  - Secondary Capture Image Storage (1.2.840.10008.5.1.4.1.1.7)
  - X-Ray Angiographic Image Storage (1.2.840.10008.5.1.4.1.1.12.1)
  - X-Ray Radiofluoroscopic Image Storage (1.2.840.10008.5.1.4.1.1.12.2)

### 対応する転送構文

- Implicit VR Little Endian: Default Transfer Syntax for DICOM (1.2.840.10008.1.2)
- Explicit VR Little Endian (1.2.840.10008.1.2.1)

### 対応する文字セット

以下の`Specific Character Set`の値に対応。

- `""` (デフォルト文字セット)
- `"ISO_IR 13"`
- `"ISO_IR 192"`
- `"ISO 2022 IR 6\ISO 2022 IR 87"`
- `"ISO 2022 IR 13\ISO 2022 IR 87"`
- `"ISO 2022 IR 6\ISO 2022 IR 13\ISO 2022 IR 87"`
