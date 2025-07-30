# Oceanus

Rustで実装されたPACSです。

## 必要な環境

- Docker
- Docker Compose
- Make

## 使い方

### 1. 開発環境の初期化

```bash
make init
```

このコマンドでDockerが起動し、開発や実行に必要な環境が整います。

### 2. アプリケーションの実行

#### 開発モード

```bash
make run
```

#### リリースモード

```bash
make run PROFILE=release
```

### 3. 開発ツール

#### コードの静的解析

```bash
make lint
```

#### コードのフォーマット

```bash
make format
```

#### テストの実行

```bash
make test
```

### 4. 環境の停止

```bash
make exit
```

## プロジェクト構成

```
oceanus/
├── docker/                 # Dockerイメージのソース
├── src/                    # Rustワークスペースのルート
│   ├── Cargo.toml          # ワークスペース設定
│   └── dicom-server/       # DICOMサーバー（バイナリクレート）
│       ├── Cargo.toml
│       ├── Makefile
│       └── src/
│           └── main.rs
├── docker-compose.yml      # Docker環境設定
├── Makefile
└── README.md
```
