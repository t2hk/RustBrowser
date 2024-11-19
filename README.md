# RustBrowser
# 概要
技術評論社の土井麻未さん著「作って学ぶ ブラウザのしくみ」のサンプルを使った Rust の学習コードである。

https://github.com/d0iasm/saba

# 環境構築

* パッケージ最新化
```
sudo apt update -y
sudo apt upgrade -y
```

* ツールチェイン設定
```rust-toolchain.toml
[toolchain]
channel = "nightly-2024-01-01"
components = [ "rustfmt", "rust-src" ]
targets = [ "x86_64-unknown-linux-gnu" ]
profile = "default"
```

* Rust インストールなど
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup show
```

* QEMU インストール
```
apt install qemu-system
```

# 第２章 URL の分解
URL のスキーマ、ホスト、ポート、パス、クエリパラメータを構造体で表現する。
文字列 URL をパースして URL 構造体を組み立てる。

テストの実行方法は以下の通り。
```
cd saba_core
cargo test
```

# 第３章 HTTP の実装
HTTP クライアントを作成し、HTTP リクエストの送受信を実装する。

テストの実施方法は以下の通り。
```
cd saba_core
cargo test
```

wasabiOS 上で動かす方法は以下の通り。
```
./run_on_wasabi.sh

# OS が起動したら、"rustbrowser" と入力して Enter を押下する。
# 以下のように HTML 文字列が表示される。

response:
HttpResponse {
    version: "HTTP/1.1",
    status_code: 200,
    reason: "OK\r",
... 省略 ...
```

テストサーバとやり取りする方法は以下の通り。
```
python3 -m http.server 8000
```

