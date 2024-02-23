use itertools::Itertools;
use lindera_core::{
    error::LinderaError,
    mode::{Mode, Penalty},
};
use lindera_dictionary::{DictionaryConfig, DictionaryKind};
use lindera_tokenizer::tokenizer::{Tokenizer, TokenizerConfig};

const LINDERA_DETAIL_PRONOUNCIATION_COLUMN: usize = 9;

pub type GomamayoResult<T, E = GomamayoError> = Result<T, E>;

#[derive(Debug)]
pub struct UnknownPronounciationError {
    pub text: String,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum GomamayoError {
    LinderaError(LinderaError),
    UnknownPronounciationError(UnknownPronounciationError),
}

impl From<LinderaError> for GomamayoError {
    fn from(value: LinderaError) -> Self {
        GomamayoError::LinderaError(value)
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
    let dictionary = DictionaryConfig {
        kind: Some(DictionaryKind::UniDic),
        path: None,
    };

    let config = TokenizerConfig {
        dictionary,
        user_dictionary: None,
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
                    d.get(LINDERA_DETAIL_PRONOUNCIATION_COLUMN)
                        .map(|p| p.to_string())
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

fn compute_ary_and_degree<S: AsRef<str>>(pronounciations: &[S]) -> (i32, i32) {
    let mut ary: i32 = 0;
    let mut degree: i32 = 0;

    for (left, right) in pronounciations
        .iter()
        .map(|s| s.as_ref().chars().collect_vec())
        .tuple_windows()
    {
        let found_degree = (1..=left.len().min(right.len()))
            .rev()
            .find(|&d| left[left.len() - d..] == right[..d]);

        if let Some(current_degree) = found_degree {
            degree = degree.max(current_degree as i32);
            ary += 1;
        }
    }

    (ary, degree)
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
            expected_pronounciations: &["セワ", "ヤキ", "キツネ", "ノ", "セン", "キツネ", "サン"],
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
            expected_pronounciations: &["ハク", "レー", "レーム"],
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
