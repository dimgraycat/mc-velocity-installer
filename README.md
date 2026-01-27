# mc-velocity-installer

Velocity を新規インストールする対話型CLIです。各ステップで確認を行い、ダウンロードした jar（ファイル名はURLのものをそのまま使用）と起動スクリプトを生成します。設定ファイルは初回起動で生成されます。

## 前提

- Java がインストール済みで `java` が `PATH` から実行できること
- ネットワークアクセスが可能であること（バージョン一覧取得・ダウンロードに使用）
- 実行ディレクトリ配下に書き込む権限があること（既定のインストール先は `./velocity`）

## 使い方

```bash
cargo run
```

jar のみ再取得:

```bash
cargo run -- --redownload-jar
```

### 対話の流れ（概要）

1. インストール先の指定（既定: 実行時のカレントディレクトリ/velocity）
2. 既存ファイルの有無確認（上書き可否）
3. Velocity バージョン一覧の表示と選択（`バージョン (type, build)` 形式）
4. 起動メモリ（Xms/Xmx）の入力（既定: 256M/512M）
5. サマリ確認後、ダウンロードと生成
6. 初回起動で `velocity.toml` を生成（生成後に設定を編集）

## 生成物

- ダウンロードした jar（ファイル名はURLのものをそのまま使用）
- `start.sh` / `start.bat`
- `velocity.service`（systemd 用ユニットファイル）

## オプション

- `--redownload-jar` は jar のみ再取得します（start.sh / start.bat の置き換えは確認後に実行）。
- `-h, --help` でヘルプを表示します。
- `-V, --version` でバージョンを表示します。

## 詳細仕様

仕様の詳細は `docs/README.md` を参照してください。
