# Oceanus

Rust で実装された PACS です。

## 必要な環境

- Rust (1.85.0 以上)
- Make

## 使い方

### 1. アプリケーションの実行

#### 開発モード

```bash
make run
```

#### リリースモード

```bash
make run PROFILE=release
```

### 2. 開発ツール

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

### 3. ビルド成果物の削除

```bash
make clean
```

## プロジェクト構成

```
oceanus/
├── src/                    # Rustワークスペースのルート
│   ├── Cargo.toml          # ワークスペース設定
│   └── dicom-server/       # DICOMサーバー（バイナリクレート）
│       ├── Cargo.toml
│       ├── Makefile
│       └── src/
│           └── main.rs
├── Makefile
└── README.md
```
