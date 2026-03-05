#![deny(clippy::all)]

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use napi_derive::napi;

use sudachi::analysis::stateful_tokenizer::StatefulTokenizer;
use sudachi::analysis::Mode;
use sudachi::config::Config;
use sudachi::dic::dictionary::JapaneseDictionary;
use sudachi::prelude::MorphemeList;

/// A single morpheme (tokenization result unit)
#[napi(object)]
pub struct Morpheme {
    /// Surface form — the substring of the original text
    pub surface: String,
    /// Part-of-speech components (6 elements: pos1, pos2, pos3, pos4, cType, cForm)
    pub part_of_speech: Vec<String>,
    /// Reading form in katakana (フリガナ)
    pub reading_form: String,
    /// Dictionary (lemma) form
    pub dictionary_form: String,
    /// Normalized form
    pub normalized_form: String,
    /// Whether this morpheme is out-of-vocabulary
    pub is_oov: bool,
    /// Begin byte offset in the original text
    pub begin: u32,
    /// End byte offset in the original text
    pub end: u32,
    /// Dictionary id (-1 for OOV words)
    pub dictionary_id: i32,
}

/// A Sudachi Japanese dictionary
#[napi]
pub struct Dictionary {
    inner: Arc<JapaneseDictionary>,
}

#[napi]
impl Dictionary {
    /// Load a dictionary from a compiled system dictionary file.
    ///
    /// @param dictPath - Path to the compiled system dictionary (.dic file)
    /// @param resourceDir - Optional path to the resource directory (containing char.def, unk.def, etc.)
    /// @param configPath - Optional path to sudachi.json config file
    #[napi(constructor)]
    pub fn new(
        dict_path: String,
        resource_dir: Option<String>,
        config_path: Option<String>,
    ) -> napi::Result<Self> {
        let config = Config::new(
            config_path.map(PathBuf::from),
            resource_dir.map(PathBuf::from),
            Some(PathBuf::from(dict_path)),
        )
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        let dict = JapaneseDictionary::from_cfg(&config)
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;

        Ok(Self {
            inner: Arc::new(dict),
        })
    }

    /// Create a reusable Tokenizer for this dictionary.
    ///
    /// @param mode - Split mode: "A" (short), "B" (middle), "C" (long, default)
    #[napi]
    pub fn create(&self, mode: Option<String>) -> napi::Result<Tokenizer> {
        Ok(Tokenizer {
            dict: self.inner.clone(),
            mode: parse_mode(mode.as_deref())?,
        })
    }

    /// Tokenize Japanese text and return morphemes.
    ///
    /// @param text - Japanese text to analyze
    /// @param mode - Split mode: "A" (short), "B" (middle), "C" (long, default)
    #[napi]
    pub fn tokenize(&self, text: String, mode: Option<String>) -> napi::Result<Vec<Morpheme>> {
        run_tokenize(&self.inner, &text, parse_mode(mode.as_deref())?)
    }
}

/// A stateful tokenizer bound to a specific dictionary and split mode
#[napi]
pub struct Tokenizer {
    dict: Arc<JapaneseDictionary>,
    mode: Mode,
}

#[napi]
impl Tokenizer {
    /// Tokenize Japanese text and return morphemes.
    ///
    /// @param text - Japanese text to analyze
    /// @param mode - Override split mode for this call only
    #[napi]
    pub fn tokenize(&self, text: String, mode: Option<String>) -> napi::Result<Vec<Morpheme>> {
        let mode = match mode {
            Some(m) => parse_mode(Some(&m))?,
            None => self.mode,
        };
        run_tokenize(&self.dict, &text, mode)
    }

    /// The split mode of this tokenizer ("A", "B", or "C")
    #[napi(getter)]
    pub fn mode(&self) -> String {
        match self.mode {
            Mode::A => "A".to_string(),
            Mode::B => "B".to_string(),
            Mode::C => "C".to_string(),
        }
    }
}

fn parse_mode(mode: Option<&str>) -> napi::Result<Mode> {
    match mode {
        None => Ok(Mode::C),
        Some(m) => {
            Mode::from_str(m).map_err(|e| napi::Error::from_reason(e.to_string()))
        }
    }
}

fn run_tokenize(
    dict: &Arc<JapaneseDictionary>,
    text: &str,
    mode: Mode,
) -> napi::Result<Vec<Morpheme>> {
    let mut tok = StatefulTokenizer::new(dict.clone(), mode);
    tok.reset().push_str(text);
    tok.do_tokenize()
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let mut list = MorphemeList::empty(dict.clone());
    list.collect_results(&mut tok)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    let mut result = Vec::with_capacity(list.len());
    for m in list.iter() {
        result.push(Morpheme {
            surface: m.surface().to_string(),
            part_of_speech: m.part_of_speech().to_vec(),
            reading_form: m.reading_form().to_string(),
            dictionary_form: m.dictionary_form().to_string(),
            normalized_form: m.normalized_form().to_string(),
            is_oov: m.is_oov(),
            begin: m.begin() as u32,
            end: m.end() as u32,
            dictionary_id: m.dictionary_id(),
        });
    }
    Ok(result)
}
