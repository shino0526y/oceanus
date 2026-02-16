# ビルドステージ
FROM --platform=$BUILDPLATFORM rust:1.91-slim-bookworm AS builder

# ターゲットアーキテクチャの指定
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    gcc-x86-64-linux-gnu \
    libc6-dev-amd64-cross \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-gnu

WORKDIR /app

# ワークスペース全体をコピー
COPY Cargo.toml Cargo.lock ./
COPY dicom-lib/Cargo.toml dicom-lib/
COPY dicom-server/Cargo.toml dicom-server/
COPY web-api/Cargo.toml web-api/

COPY . .

# クロスコンパイルの実行
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL

RUN cargo build --release --target x86_64-unknown-linux-gnu -p dicom-server -p web-api

# 実行ステージのベース
FROM --platform=linux/amd64 gcr.io/distroless/cc-debian12 AS runtime

# DICOM サーバー ステージ
FROM runtime AS dicom-server
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/dicom-server /usr/local/bin/
EXPOSE 104
CMD ["dicom-server"]

# Web API ステージ
FROM runtime AS web-api
WORKDIR /app
COPY --from=builder /app/target/x86_64-unknown-linux-gnu/release/web-api /usr/local/bin/
EXPOSE 8080
CMD ["web-api"]
