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

# 第２章 URL　の分解
URL のスキーマ、ホスト、ポート、パス、クエリパラメータを構造体で表現する。
文字列 URL をパースして URL 構造体を組み立てる。

テストの実行方法は以下のとおり。
```
cd saba_core
cargo test
```

# 第３章 HTTP　の実装

