# ビルドステージ
FROM rust:1.91-slim-bookworm AS builder

# データベース接続情報の引数
ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

# ビルド依存関係のインストール
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ワークスペース全体をコピー
COPY Cargo.toml Cargo.lock ./
COPY dicom-lib/Cargo.toml dicom-lib/
COPY dicom-server/Cargo.toml dicom-server/
COPY web-api/Cargo.toml web-api/

COPY . .

# 両方のバイナリをビルド
RUN cargo build --release -p dicom-server -p web-api

# 実行ステージのベース
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# DICOM サーバー ステージ
FROM runtime AS dicom-server
WORKDIR /app
COPY --from=builder /app/target/release/dicom-server /usr/local/bin/
EXPOSE 104
CMD ["dicom-server"]

# Web API ステージ
FROM runtime AS web-api
WORKDIR /app
COPY --from=builder /app/target/release/web-api /usr/local/bin/
EXPOSE 8080
CMD ["web-api"]
