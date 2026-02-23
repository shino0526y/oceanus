# ビルドステージ
FROM --platform=$BUILDPLATFORM rust:1.91-slim-bookworm AS builder

ARG TARGETARCH

# ターゲットアーキテクチャに応じたツールチェーンのインストール
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && case "$TARGETARCH" in \
    "amd64") \
    apt-get install -y gcc-x86-64-linux-gnu libc6-dev-amd64-cross; \
    rustup target add x86_64-unknown-linux-gnu; \
    ;; \
    "arm64") \
    apt-get install -y gcc-aarch64-linux-gnu libc6-dev-arm64-cross; \
    rustup target add aarch64-unknown-linux-gnu; \
    ;; \
    esac \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ワークスペース全体をコピー
COPY Cargo.toml Cargo.lock ./
COPY dicom-lib/Cargo.toml dicom-lib/
COPY dicom-server/Cargo.toml dicom-server/
COPY web-api/Cargo.toml web-api/

COPY . .

# クロスコンパイルの実行
ENV PKG_CONFIG_ALLOW_CROSS=1

ARG DATABASE_URL
ENV DATABASE_URL=$DATABASE_URL
ENV SQLX_OFFLINE=true

RUN case "$TARGETARCH" in \
    "amd64") \
    export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc; \
    cargo build --release --target x86_64-unknown-linux-gnu -p dicom-server -p web-api; \
    mkdir -p /app/target/release-bin && \
    cp /app/target/x86_64-unknown-linux-gnu/release/dicom-server /app/target/release-bin/ && \
    cp /app/target/x86_64-unknown-linux-gnu/release/web-api /app/target/release-bin/; \
    ;; \
    "arm64") \
    export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc; \
    cargo build --release --target aarch64-unknown-linux-gnu -p dicom-server -p web-api; \
    mkdir -p /app/target/release-bin && \
    cp /app/target/aarch64-unknown-linux-gnu/release/dicom-server /app/target/release-bin/ && \
    cp /app/target/aarch64-unknown-linux-gnu/release/web-api /app/target/release-bin/; \
    ;; \
    esac

# 実行ステージのベース
FROM gcr.io/distroless/cc-debian12 AS runtime

# DICOM サーバー ステージ
FROM runtime AS dicom-server
WORKDIR /app
COPY --from=builder /app/target/release-bin/dicom-server /usr/local/bin/
EXPOSE 104
CMD ["dicom-server"]

# Web API ステージ
FROM runtime AS web-api
WORKDIR /app
COPY --from=builder /app/target/release-bin/web-api /usr/local/bin/
EXPOSE 8080
CMD ["web-api"]
