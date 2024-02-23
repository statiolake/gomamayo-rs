use std::io::{self, Write};

use itertools::Itertools;
use lindera_core::{
    error::LinderaError,
    mode::{Mode, Penalty},
};
use lindera_dictionary::{DictionaryConfig, DictionaryKind, UserDictionaryConfig};
use lindera_tokenizer::tokenizer::{Tokenizer, TokenizerConfig};
use tempfile::Builder;

const LINDERA_DETAIL_READING_COLUMN: usize = 6;
const LINDERA_DETAIL_PRONOUNCIATION_COLUMN: usize = 9;

pub type GomamayoResult<T, E = GomamayoError> = Result<T, E>;

#[derive(Debug)]
pub struct UnknownPronounciationError {
    pub text: String,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum GomamayoError {
    IoError(io::Error),
    LinderaError(LinderaError),
    UnknownPronounciationError(UnknownPronounciationError),
}

impl From<LinderaError> for GomamayoError {
    fn from(value: LinderaError) -> Self {
        GomamayoError::LinderaError(value)
    }
}

impl From<io::Error> for GomamayoError {
    fn from(value: io::Error) -> Self {
        GomamayoError::IoError(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Gomamayo {
    pub kind: Option<GomamayoKind>,
    pub pronounciations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct GomamayoKind {
    pub ary: i32,
    pub degree: i32,
}

fn tokenize_to_pronounciations(input: &str) -> GomamayoResult<Vec<String>> {
    // ユーザー辞書を一時ファイルに書き出す (Linderaではファイルを指定する必要があるため)
    let mut user_jisyo_temp_file = Builder::new().suffix(".csv").tempfile()?;
    user_jisyo_temp_file.write_all(include_bytes!("./user_jisyo.csv"))?;

    let dictionary = DictionaryConfig {
        kind: Some(DictionaryKind::UniDic),
        path: None,
    };

    let user_dictionary = Some(UserDictionaryConfig {
        kind: Some(DictionaryKind::UniDic),
        path: user_jisyo_temp_file.path().to_owned(),
    });

    let config = TokenizerConfig {
        dictionary,
        user_dictionary,
        mode: Mode::Decompose(Penalty::default()),
    };

    let tokenizer = Tokenizer::from_config(config)?;
    let mut tokens = tokenizer.tokenize(input)?;

    let pronounciations = tokens
        .iter_mut()
        .map(|token| {
            token
                .get_details()
                .and_then(|d| {
                    if let Some(p) = d.get(LINDERA_DETAIL_PRONOUNCIATION_COLUMN) {
                        if *p != "*" {
                            return Some(p.to_string());
                        }
                    }

                    if let Some(r) = d.get(LINDERA_DETAIL_READING_COLUMN) {
                        if *r != "*" {
                            return Some(r.to_string());
                        }
                    }

                    None
                })
                .ok_or_else(|| {
                    GomamayoError::UnknownPronounciationError(UnknownPronounciationError {
                        text: token.text.to_string(),
                    })
                })
        })
        .collect::<GomamayoResult<Vec<_>, _>>()?;

    Ok(pronounciations)
}

fn into_moras(pronounciation: &str) -> Vec<String> {
    let mut moras = vec![];
    let mut curr = String::new();

    for c in pronounciation.chars() {
        if !"ャュョァィゥェォ".contains(c) && !curr.is_empty() {
            moras.push(curr);
            curr = String::new();
        }

        curr.push(c);
    }

    if !curr.is_empty() {
        moras.push(curr);
    }

    moras
}

fn compute_ary_and_degree<S: AsRef<str>>(pronounciations: &[S]) -> (i32, i32) {
    let mut ary: i32 = 0;
    let mut max_degree: i32 = 0;

    for (left, right) in pronounciations
        .iter()
        .map(|s| into_moras(s.as_ref()))
        .tuple_windows()
    {
        let degree = (1..=left.len().min(right.len()))
            .rev()
            .find(|&d| left[left.len() - d..] == right[..d]);

        if let Some(degree) = degree {
            max_degree = max_degree.max(degree as i32);
            ary += 1;
        }
    }

    (ary, max_degree)
}

pub fn analyze(input: &str) -> GomamayoResult<Gomamayo> {
    let pronounciations = tokenize_to_pronounciations(input)?;

    let (ary, degree) = compute_ary_and_degree(&pronounciations);
    let kind = if ary > 0 {
        Some(GomamayoKind { ary, degree })
    } else {
        None
    };

    Ok(Gomamayo {
        kind,
        pronounciations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase {
        input: &'static str,
        expected_pronounciations: &'static [&'static str],
        expected_ary: i32,
        expected_degree: i32,
    }

    const TEST_CASES: &[TestCase] = &[
        TestCase {
            input: "ゴママヨ",
            expected_pronounciations: &["ゴマ", "マヨ"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "安田大サーカス",
            expected_pronounciations: &["ヤスダ", "ダイ", "サーカス"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "福山雅治さん",
            expected_pronounciations: &["フクヤマ", "マサハル", "サン"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "世話やきキツネの仙狐さん",
            expected_pronounciations: &["セワ", "ヤキ", "キツネ", "ノ", "センコ", "サン"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "サイレンススズカ",
            expected_pronounciations: &["サイレンス", "スズカ"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "長期金利",
            expected_pronounciations: &["チョーキ", "キンリ"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "博麗霊夢",
            expected_pronounciations: &["ハクレー", "レーム"],
            expected_ary: 1,
            expected_degree: 2,
        },
        TestCase {
            input: "株式公開買い付け",
            expected_pronounciations: &["カブシキ", "コーカイ", "カイツケ"],
            expected_ary: 1,
            expected_degree: 2,
        },
        TestCase {
            input: "自己肯定",
            expected_pronounciations: &["ジコ", "コーテー"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "千載一遇",
            expected_pronounciations: &["センザイ", "イチグー"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "投資信託",
            expected_pronounciations: &["トーシ", "シンタク"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "消火活動",
            expected_pronounciations: &["ショーカ", "カツドー"],
            expected_ary: 1,
            expected_degree: 1,
        },
        TestCase {
            input: "銀行口座",
            expected_pronounciations: &["ギンコー", "コーザ"],
            expected_ary: 1,
            expected_degree: 2,
        },
        TestCase {
            input: "診療受付",
            expected_pronounciations: &["シンリョー", "ウケツケ"],
            expected_ary: 0,
            expected_degree: 0,
        },
        TestCase {
            input: "パパイヤ",
            expected_pronounciations: &["パパイヤ"],
            expected_ary: 0,
            expected_degree: 0,
        },
        TestCase {
            input: "パパイヤジュース",
            expected_pronounciations: &["パパイヤ", "ジュース"],
            expected_ary: 0,
            expected_degree: 0,
        },
        TestCase {
            input: "無性生殖",
            expected_pronounciations: &["ムセー", "セーショク"],
            expected_ary: 1,
            expected_degree: 2,
        },
        TestCase {
            input: "部分分数分解",
            expected_pronounciations: &["ブブン", "ブンスー", "ブンカイ"],
            expected_ary: 1,
            expected_degree: 2,
        },
        TestCase {
            input: "モバイルルータ端末",
            expected_pronounciations: &["モバイル", "ルータ", "タンマツ"],
            expected_ary: 2,
            expected_degree: 1,
        },
        TestCase {
            input: "太鼓公募募集終了",
            expected_pronounciations: &["タイコ", "コーボ", "ボシュー", "シューリョー"],
            expected_ary: 3,
            expected_degree: 2,
        },
        TestCase {
            input: "多項高次ゴママヨ",
            expected_pronounciations: &["タコー", "コージ", "ゴマ", "マヨ"],
            expected_ary: 2,
            expected_degree: 2,
        },
        TestCase {
            input: "オレンジジュース",
            expected_pronounciations: &["オレンジ", "ジュース"],
            expected_ary: 0,
            expected_degree: 0,
        },
    ];

    #[test]
    fn correct_tokenize() {
        for case in TEST_CASES {
            assert_eq!(
                tokenize_to_pronounciations(case.input).unwrap(),
                case.expected_pronounciations,
            );
        }
    }

    #[test]
    fn test_into_moras() {
        assert_eq!(
            into_moras("オレンジジュース"),
            ["オ", "レ", "ン", "ジ", "ジュ", "ー", "ス"]
        );
        assert_eq!(into_moras("シャワー"), ["シャ", "ワ", "ー"]);
        assert_eq!(into_moras("ボシュー"), ["ボ", "シュ", "ー"]);
        assert_eq!(into_moras("シューリョー"), ["シュ", "ー", "リョ", "ー"]);
    }

    #[test]
    fn correct_ary_degree() {
        for case in TEST_CASES {
            let (ary, degree) = compute_ary_and_degree(case.expected_pronounciations);
            assert_eq!(
                ary, case.expected_ary,
                "wrong ary for {:?}",
                case.expected_pronounciations
            );
            assert_eq!(
                degree, case.expected_degree,
                "wrong degree for {:?}",
                case.expected_pronounciations
            );
        }
    }
}
