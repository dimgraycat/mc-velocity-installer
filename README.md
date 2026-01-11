# mc-velocity-installer

Velocity を新規インストールする対話型CLIです。各ステップで確認を行い、`velocity.jar` と設定ファイル、起動スクリプトを生成します。

## 前提

- Java がインストール済みで `java` が `PATH` から実行できること
- ネットワークアクセスが可能であること（バージョン一覧取得・ダウンロードに使用）
- `/opt/` 配下に書き込む権限があること（既定のインストール先）

## 使い方

```bash
cargo run
```

### 対話の流れ（概要）

1. インストール先の指定（既定: `/opt/`）
2. 既存ファイルの有無確認（上書き可否）
3. Velocity バージョン一覧の表示と選択（`バージョン (type)` 形式）
4. 主要設定の入力
5. 起動メモリ（Xms/Xmx）の入力（既定: 256M/512M）
6. サマリ確認後、ダウンロードと生成

### 主要設定で入力する項目

- リッスンアドレス/ポート（`bind`）
- MOTD（`motd`）
- プレイヤー数表示（`show-max-players`）
- オンラインモード（`online-mode`）
- 鍵認証強制（`force-key-authentication`）
- 転送モード（`player-info-forwarding-mode`）
  - `none` / `legacy` / `bungeeguard` / `modern`
- 共有シークレット（`forwarding.secret` に保存。`bungeeguard`/`modern` の場合のみ）
- バックエンドサーバ定義（`servers`）
- 接続順序（`try`）

## 生成物

- `velocity.jar`
- `velocity.toml`
- `forwarding.secret`（必要時のみ）
- `start.sh` / `start.bat`

## オプション

- `--update` は未対応です。指定された場合は案内メッセージを出して終了します。

## 詳細仕様

仕様の詳細は `docs/README.md` を参照してください。

