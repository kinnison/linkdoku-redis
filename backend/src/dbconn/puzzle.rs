//! Puzzle data in the redis database

use std::io::Cursor;

use linkdoku_common::{PuzzleState, Visibility};
use serde::{Deserialize, Serialize};

/// Puzzles are the core data which most users of Linkdoku care about
///
/// A puzzle is owned by a role, but may have multiple other roles able
/// to access it.
///
/// Redis keys:
///
/// * `puzzle:{uuid}` hash containing core puzzle data
/// * `puzzle:byname` hash containing normalised short-name to puzzle UUID mapping
///
/// Note: a large amount of the puzzle data is actually a compressed serialised JSON object
#[derive(Debug, Serialize, Deserialize)]
pub struct Puzzle {
    uuid: String,
    owner: String,
    short_name: String,
    display_name: String,
    visibility: Visibility,
    visibility_date: Option<String>,
    states: Vec<PuzzleState>,
}

impl Puzzle {
    /// UUID of the puzzle
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// Owner of the puzzle
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// The short name of the puzzle
    pub fn short_name(&self) -> &str {
        self.short_name.as_ref()
    }

    /// The display name of the puzzle
    pub fn display_name(&self) -> &str {
        self.display_name.as_ref()
    }

    /// The visibility of this puzzle
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// The visibility date change of this puzzle
    pub fn visibility_date(&self) -> Option<&str> {
        self.visibility_date.as_deref()
    }

    /// The states of the puzzle
    pub fn states(&self) -> &[PuzzleState] {
        self.states.as_ref()
    }

    /// Load a puzzle from the database
    pub(crate) fn from_list(uuid: &str, mut kvs: impl Iterator<Item = String>) -> Self {
        let mut ret = Self {
            uuid: uuid.to_string(),
            owner: String::new(),
            short_name: String::new(),
            display_name: String::new(),
            visibility: Visibility::Restricted,
            visibility_date: None,
            states: Vec::new(),
        };
        while let Some(key) = kvs.next() {
            if let Some(value) = kvs.next() {
                match key.as_str() {
                    "owner" => ret.owner = value,
                    "short_name" => ret.short_name = value,
                    "display_name" => ret.display_name = value,
                    "visibility" => match value.as_str() {
                        "restricted" => ret.visibility = Visibility::Restricted,
                        "public" => ret.visibility = Visibility::Public,
                        "published" => ret.visibility = Visibility::Published,
                        _ => tracing::warn!("Unknown visibility string decoding Puzzle: {}", value),
                    },
                    "visibility_date" => {
                        if value.is_empty() {
                            ret.visibility_date = None;
                        } else {
                            ret.visibility_date = Some(value);
                        }
                    }
                    "states" => ret.states = Self::decompress_state(&value),
                    _ => tracing::warn!("Unknown kv pair decoding Puzzle: {}={}", key, value),
                }
            }
        }
        ret
    }

    pub(super) fn compress_states(states: &[PuzzleState]) -> String {
        let mut out = Vec::new();
        let mut writer = xz2::write::XzEncoder::new(&mut out, 9);
        serde_json::to_writer(&mut writer, states).expect("Unable to load states?");
        drop(writer);
        base64::encode(&out)
    }

    fn decompress_state(states: &str) -> Vec<PuzzleState> {
        let bytes = base64::decode(states).expect("Unable to b64 decode states");
        let reader = xz2::read::XzDecoder::new(Cursor::new(bytes));
        serde_json::from_reader(reader).expect("Unable to decode puzzle states")
    }

    pub fn create_uuid(owner: &str, short_name: &str) -> String {
        format!(
            "{:x}",
            md5::compute(format!("puzzle:owner:{}:name:{}", owner, short_name))
        )
    }

    pub fn as_api_puzzle(&self, is_owner: bool) -> linkdoku_common::Puzzle {
        let mut ret = linkdoku_common::Puzzle {
            uuid: self.uuid().to_string(),
            owner: self.owner().to_string(),
            short_name: self.short_name().to_string(),
            display_name: self.display_name().to_string(),
            visibility: self.visibility,
            visibility_changed: self.visibility_date().map(String::from),
            states: Vec::new(),
        };
        for state in &self.states {
            match state.visibility {
                Visibility::Restricted if !is_owner => continue,
                Visibility::Restricted => ret.states.push(state.clone()),
                Visibility::Public | Visibility::Published => ret.states.push(state.clone()),
            }
        }
        ret
    }
}

impl From<linkdoku_common::Puzzle> for Puzzle {
    fn from(input: linkdoku_common::Puzzle) -> Self {
        let linkdoku_common::Puzzle {
            owner,
            display_name,
            short_name,
            visibility,
            uuid,
            visibility_changed,
            states,
        } = input;
        Self {
            uuid,
            owner,
            short_name,
            display_name,
            visibility,
            visibility_date: visibility_changed,
            states,
        }
    }
}
