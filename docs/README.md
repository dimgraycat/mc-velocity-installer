# ドキュメント概要

この `docs` は本リポジトリの仕様・運用の単一の参照元です。変更を加えた場合は、必ず内容を更新してください。

## プロジェクト概要

- プロジェクト名: `mc-velocity-installer`
- 種別: Rust バイナリ（対話型CLI）
- Rust Edition: 2024
- 依存関係: なし（`Cargo.toml` に外部crateなし）
- 実装状況: `cargo run` で `Hello, world!` を標準出力（仕様は本ドキュメントに準拠して実装する）

## 仕様

### 目的

- Velocity の新規インストールとアップデートを対話式で行う
- 操作の各段階でユーザーに確認を求める（Y/n）

### 前提

- Java は既にインストールされ、`java` が `PATH` で実行可能
- ネットワークアクセスが可能（バージョン一覧取得とダウンロードに必要）

### モードと引数

- 新規インストール: 引数なしで起動した場合の既定モード
- アップデート: `--update` の指定が必須
  - `--update` が無い場合は更新処理を行わず、再実行を案内する

### バージョン一覧の取得元

- URL: `https://minedeck.github.io/jars/velocity.json`
- 利用する項目:
  - `data` のキーをバージョン一覧として扱う
  - 各バージョンの `url` をダウンロード先として使用する
  - `checksum.sha256` でダウンロードの整合性を検証する
  - 一覧表示は `バージョン (type)` 形式にする（例: `3.4.0-SNAPSHOT (stable)`）

### 対話フロー（都度確認）

新規インストール:

1. ツール概要と注意事項の表示
2. インストール先ディレクトリの指定（既定: `/opt/`）
3. 既存インストールの検出と確認（上書き/中止）
4. Velocity バージョン選択（一覧から選択）
5. 主要設定の入力（下記参照）
6. 起動スクリプト設定（メモリ量など）
7. 実行前サマリ表示と最終確認
8. ダウンロード → チェックサム検証 → 配置
9. 設定ファイル生成、起動スクリプト生成
10. 完了メッセージと次の手順の案内

アップデート（`--update` 必須）:

1. アップデートモードの説明と注意事項の表示
2. 既存インストール先の指定
3. 既存ファイル検出（`velocity.jar`/`velocity.toml`）と確認
4. 更新対象バージョンの選択
5. 更新内容の確認
6. ダウンロード → チェックサム検証 → `velocity.jar` の置換
7. 起動スクリプトの再生成有無の確認（未存在時は作成）
8. 完了メッセージと次の手順の案内

### 主要設定項目

以下を「主要設定」として対話で確認する。

- リッスンアドレス/ポート（`bind`）
- MOTD（`motd`）
- 最大人数（`max-players`）
- プレイヤー数表示（`show-max-players`）
- オンラインモード（`online-mode`）
- 転送モード（`player-info-forwarding-mode`）
  - `none` / `legacy` / `bungeeguard` / `velocity`
- 共有シークレット（`forwarding-secret`）
- 鍵認証強制（`force-key-authentication`）
- バックエンドサーバ定義（`servers`）
  - 例: `lobby = "127.0.0.1:30066"`
- 接続順序（`try`）
  - 例: `["lobby"]`

### 生成物

- `velocity.jar` : ダウンロードした本体（更新時は置換）
- `velocity.toml` : 設定ファイル（新規作成、更新時は保持）
- `start.sh` / `start.bat` : 起動スクリプト

### 起動スクリプト

- `start.sh` と `start.bat` を必ず生成する
- 実行内容は以下を基本とする（メモリ値は対話で指定）
  - `java -Xms{min} -Xmx{max} -jar velocity.jar`
- `start.sh` は実行権限を付与する
- 既定メモリ: `-Xms256M -Xmx512M`

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
