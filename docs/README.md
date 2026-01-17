# ドキュメント概要

この `docs` は本リポジトリの仕様・運用の単一の参照元です。変更を加えた場合は、必ず内容を更新してください。

## プロジェクト概要

- プロジェクト名: `mc-velocity-installer`
- 種別: Rust バイナリ（対話型CLI）
- Rust Edition: 2024
- 依存関係: `reqwest`, `serde`, `serde_json`, `sha2`
- 実装状況: 仕様に基づく対話型インストーラを実装

## 仕様

### 目的

- Velocity の新規インストールを対話式で行う
- 操作の各段階でユーザーに確認を求める（Y/n）
- アップデート機能は別途対応（本ツールでは未対応）

### 前提

- Java は既にインストールされ、`java` が `PATH` で実行可能
- ネットワークアクセスが可能（バージョン一覧取得とダウンロードに必要）

### モードと引数

- 新規インストールのみ対応（引数なしで起動）
- `--setup` は `velocity.toml` を対話式で編集する
  - 既定パス: 実行時のカレントディレクトリ/velocity/velocity.toml
  - 項目ごとにスキップ/削除が可能
- `--update` は未対応として案内メッセージを出して終了する

### バージョン一覧の取得元

- URL: `https://minedeck.github.io/jars/velocity.json`
- 利用する項目:
  - `data` のキーをバージョン一覧として扱う
  - 各バージョンの `url` をダウンロード先として使用する
  - `checksum.sha256` でダウンロードの整合性を検証する
  - 一覧表示は `バージョン (type)` 形式にする（例: `3.4.0-SNAPSHOT (stable)`）

### 対話フロー（都度確認）

1. ツール概要と注意事項の表示
2. インストール先ディレクトリの指定（既定: 実行時のカレントディレクトリ/velocity）
3. 既存インストールの検出と確認（上書き/中止）
4. Velocity バージョン選択（一覧から選択）
5. 起動スクリプト設定（メモリ量など）
6. 実行前サマリ表示と最終確認
7. ダウンロード → チェックサム検証 → 配置
8. 起動スクリプト生成
9. systemd ユニットファイル生成（`velocity.service`）
10. 完了メッセージと次の手順の案内

### 対話フロー（--setup）

1. `velocity.toml` のパス確認（既定: 実行時のカレントディレクトリ/velocity/velocity.toml）
2. 主要設定の各項目をスキップ/変更/削除で選択
3. 変更点を表示
4. 保存確認

### 主要設定項目

`velocity.toml` に含まれる主な設定例。初回起動後に手動編集または `--setup` で編集する。

- リッスンアドレス/ポート（`bind`）
- MOTD（`motd`）
- プレイヤー数表示（`show-max-players`）
- オンラインモード（`online-mode`）
- 転送モード（`player-info-forwarding-mode`）
  - `none` / `legacy` / `bungeeguard` / `modern`
- 共有シークレット（`forwarding.secret` に保存）
  - `bungeeguard` / `modern` の場合に必須（手動で作成）
- 鍵認証強制（`force-key-authentication`）
- バックエンドサーバ定義（`servers`）
  - 例: `lobby = "127.0.0.1:30066"`
- 接続順序（`try`）
  - 例: `["lobby"]`

### 生成物

- ダウンロードした jar（ファイル名はURLのものをそのまま使用）
- `start.sh` / `start.bat` : 起動スクリプト
- `velocity.service` : systemd 用ユニットファイル

`velocity.toml` はダウンロードした jar の初回起動で生成されるため、インストール時には作成しない。

### 起動スクリプト

- `start.sh` と `start.bat` を必ず生成する
- 実行内容は以下を基本とする（メモリ値は対話で指定）
  - `java -Xms{min} -Xmx{max} -jar {downloaded-jar}`
- `start.sh` は実行権限を付与する
- 既定メモリ: `-Xms256M -Xmx512M`

### systemd ユニットファイル

- `velocity.service` をインストール先に生成する
- `WorkingDirectory` はインストール先ディレクトリ
- `ExecStart` は `start.sh` を実行する
- `User` / `Group` は実行ユーザー（環境変数 `USER`）を使う
- ログは journald に出力する（`journalctl` で確認）
- 起動制限は `StartLimitIntervalSec=600` / `StartLimitBurst=6`

## 実行・ビルド・検証

必須の検証:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

基本コマンド:

```bash
cargo run
cargo build
cargo test
```

rust-analyzer はエディタ統合での解析が必須です。未設定の場合は導入してください。

## ディレクトリ構成

- `src/main.rs` : エントリポイント
- `Cargo.toml` : パッケージメタデータ（依存関係を含む）
- `Cargo.lock` : 依存関係のロックファイル
- `docs/coding_rules.md` : コーディングルール
- `docs/README.md` : 仕様・運用の単一参照元

## 変更時のルール

- 仕様や挙動を変えたら `docs/README.md` を更新する
- コーディング規約は `docs/coding_rules.md` に集約する
- 依存関係の追加・更新は `Cargo.toml` の変更理由を本ドキュメントに追記する
- 動作が変わる場合はテスト追加を検討する

## LLM/開発者向けの前提

- 本ドキュメントが最初の参照先
- ドキュメントと実装に差異がある場合は、ドキュメント更新を優先して整合させる
