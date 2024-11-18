# RustBrowser
# 概要
技術評論社の土井麻未さん著「作って学ぶ ブラウザのしくみ」のサンプルを使った Rust の学習コードである。

https://github.com/d0iasm/saba

#　第２章 URL　の分解
URL のスキーマ、ホスト、ポート、パス、クエリパラメータを構造体で表現する。
文字列 URL をパースして URL 構造体を組み立てる。

テストの実行方法は以下のとおり。
```
cd saba_core
cargo test
```

#　第３章 HTTP　の実装

