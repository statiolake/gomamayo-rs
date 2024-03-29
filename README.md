# gomamayo

ゴママヨを検知します。

## 使い方 (バイナリ実行)

```
cargo run ゴママヨ 太鼓公募募集終了 オレンジジュース
ゴママヨ: 1項1次のゴママヨです。
太鼓公募募集終了: 3項2次のゴママヨです。
オレンジジュース: ゴママヨではありません。
```

## 使い方 (ライブラリとして)

あなたのプロジェクトの Cargo.toml の `[dependencies]` 欄に以下のように追記してください。

```
[dependencies]
gomamayo = { git = "https://github.com/statiolake/gomamayo-rs" }
```

`gomamayo::analyze()` を使って解析することができます。

## テスト

参考サイトに載っていたいくつかの例はユニットテストに含まれています。
(テスト内容は src/lib.rs を参照のこと)

```
cargo test
```

## 参考

- https://thinaticsystem.com/glossary/gomamayo
- https://3qua9la-notebook.hatenablog.com/entry/2021/04/10/220317
- https://github.com/jugesuke/gomamayo/blob/master/gomamayo.go
