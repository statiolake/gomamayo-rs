use std::env;

use gomamayo::{GomamayoError, GomamayoKind, UnknownPronounciationError};

fn main() {
    for input in env::args().skip(1) {
        let input = input.trim();
        let gomamayo = match gomamayo::analyze(input) {
            Ok(gomamayo) => gomamayo,
            Err(GomamayoError::LinderaError(e)) => {
                eprintln!("Error: 入力を分かち書きできませんでした: {:?}。", e);
                return;
            }
            Err(GomamayoError::UnknownPronounciationError(UnknownPronounciationError { text })) => {
                eprintln!("Error: 単語の読み方を取得できませんでした: {text}");
                return;
            }
            Err(e) => {
                eprintln!("Error: 不明なエラーが発生しました: {:?}", e);
                return;
            }
        };

        if let Some(GomamayoKind { ary, degree }) = gomamayo.kind {
            println!("{input}: {ary}項{degree}次のゴママヨです。",);
        } else {
            println!("{input}: ゴママヨではありません。",);
        }
    }
}
