# ビルドステージ
FROM --platform=$BUILDPLATFORM rust:1.91-slim-bookworm AS builder

ARG TARGETARCH

# ターゲットアーキテクチャに応じたツールチェーンのインストール
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && case "$TARGETARCH" in \
    "amd64") CROSS_GCC="gcc-x86-64-linux-gnu" CROSS_LIBC="libc6-dev-amd64-cross" ;; \
    "arm64") CROSS_GCC="gcc-aarch64-linux-gnu" CROSS_LIBC="libc6-dev-arm64-cross" ;; \
    esac \
    && apt-get install -y $CROSS_GCC $CROSS_LIBC \
    && rustup target add $(echo $TARGETARCH | sed -e 's/amd64/x86_64-unknown-linux-gnu/' -e 's/arm64/aarch64-unknown-linux-gnu/') \
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

ENV SQLX_OFFLINE=true

RUN case "$TARGETARCH" in \
    "amd64") RUST_TARGET="x86_64-unknown-linux-gnu"   LINKER="x86_64-linux-gnu-gcc"   ;; \
    "arm64") RUST_TARGET="aarch64-unknown-linux-gnu" LINKER="aarch64-linux-gnu-gcc" ;; \
    esac \
    && export CARGO_TARGET_$(echo $RUST_TARGET | tr 'a-z-' 'A-Z_')_LINKER=$LINKER \
    && cargo build --release --target $RUST_TARGET -p dicom-server -p web-api \
    && mkdir -p /app/target/release-bin \
    && cp /app/target/$RUST_TARGET/release/dicom-server /app/target/$RUST_TARGET/release/web-api /app/target/release-bin/

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
