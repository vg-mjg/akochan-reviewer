use crate::{KyokuFilter, Pai};

use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json as json;
use serde_json::{Result, Value};
use serde_tuple::{Deserialize_tuple as DeserializeTuple, Serialize_tuple as SerializeTuple};

/// The overview structure of log in tenhou.net/6 format.
#[derive(Debug, Clone)]
pub struct Log {
    pub names: [String; 4],
    pub game_length: GameLength,
    pub has_aka: bool,
    pub kyokus: Vec<Kyoku>,
}

#[derive(Debug, Clone, Copy)]
pub enum GameLength {
    Hanchan = 0,
    Tonpuu = 4,
}

impl fmt::Display for GameLength {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameLength::Hanchan => write!(f, "半荘"),
            GameLength::Tonpuu => write!(f, "東風"),
        }
    }
}

pub mod kyoku {
    use super::*;

    /// Contains infomation about a kyoku.
    #[derive(Debug, Clone)]
    pub struct Kyoku {
        pub meta: Meta,
        pub scoreboard: [i32; 4],
        pub dora_indicators: Vec<Pai>,
        pub ura_indicators: Vec<Pai>,
        pub action_tables: [ActionTable; 4],
        pub end_status: EndStatus,
    }

    #[derive(Debug, Clone, SerializeTuple, DeserializeTuple)]
    pub struct Meta {
        pub kyoku_num: u8,
        pub honba: u8,
        pub kyotaku: u8,
    }

    #[derive(Debug, Clone)]
    pub enum EndStatus {
        Hora { details: Vec<HoraDetail> },
        Ryukyoku { score_deltas: [i32; 4] },
    }

    #[derive(Debug, Clone, Default)]
    pub struct HoraDetail {
        pub who: u8,
        pub target: u8,
        pub score_deltas: [i32; 4],
    }
}

pub use kyoku::Kyoku;

/// A group of "配牌", "取" and "出", describing a player's
/// gaming status and actions throughout a kyoku.
#[derive(Debug, Clone)]
pub struct ActionTable {
    pub haipai: [Pai; 13],
    pub takes: Vec<ActionItem>,
    pub discards: Vec<ActionItem>,
}

/// An item corresponding to each elements in "配牌", "取" and "出".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionItem {
    Pai(Pai),
    Tsumogiri(u8), // must be 60
    Naki(String),
}

mod json_scheme {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(untagged)]
    pub(super) enum ResultItem {
        Status(String),
        ScoreDeltas([i32; 4]),
        HoraDetail(Vec<Value>),
    }

    #[derive(Debug, Clone, SerializeTuple, DeserializeTuple)]
    pub(super) struct Kyoku {
        pub(super) meta: kyoku::Meta,
        pub(super) scoreboard: [i32; 4],
        pub(super) dora_indicators: Vec<Pai>,
        pub(super) ura_indicators: Vec<Pai>,

        pub(super) haipai_0: [Pai; 13],
        pub(super) takes_0: Vec<ActionItem>,
        pub(super) discards_0: Vec<ActionItem>,

        pub(super) haipai_1: [Pai; 13],
        pub(super) takes_1: Vec<ActionItem>,
        pub(super) discards_1: Vec<ActionItem>,

        pub(super) haipai_2: [Pai; 13],
        pub(super) takes_2: Vec<ActionItem>,
        pub(super) discards_2: Vec<ActionItem>,

        pub(super) haipai_3: [Pai; 13],
        pub(super) takes_3: Vec<ActionItem>,
        pub(super) discards_3: Vec<ActionItem>,

        pub(super) results: Vec<ResultItem>,
    }

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    #[serde(default)]
    pub(super) struct Rule {
        pub(super) disp: String,
        pub(super) aka: u8,
        pub(super) aka51: u8,
        pub(super) aka52: u8,
        pub(super) aka53: u8,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Log {
        #[serde(rename = "log")]
        pub(super) logs: Vec<Kyoku>,
        #[serde(rename = "name")]
        pub(super) names: [String; 4],
        pub(super) rule: Rule,

        #[serde(skip_serializing_if = "Option::is_none")]
        pub(super) ratingc: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(super) lobby: Option<i32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(super) dan: Option<Vec<String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(super) rate: Option<Vec<f64>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub(super) sx: Option<Vec<String>>,
    }

    #[derive(Debug, Serialize)]
    pub struct PartialLog<'a> {
        #[serde(flatten)]
        pub(super) parent: &'a Log,

        #[serde(rename = "log")]
        pub(super) logs: &'a [Kyoku],
    }
}

pub use json_scheme::{Log as RawLog, PartialLog as RawPartialLog};

impl RawLog {
    pub fn get_names(&self) -> &[String; 4] {
        &self.names
    }

    #[inline]
    pub fn hide_names(&mut self) {
        self.names
            .iter_mut()
            .zip('A'..='D')
            .for_each(|(name, alias)| {
                name.clear();
                name.push(alias);
                name.push_str("さん");
            });
    }

    #[inline]
    pub fn filter_kyokus(&mut self, kyoku_filter: &KyokuFilter) {
        self.logs
            .retain(|l| kyoku_filter.test(l.meta.kyoku_num, l.meta.honba))
    }

    /// Split one raw tenhou.net/6 log into many by kyokus.
    pub fn split_by_kyoku(&self) -> Vec<RawPartialLog<'_>> {
        let mut ret = vec![];

        for kyoku in self.logs.chunks(1) {
            let kyoku_log = RawPartialLog {
                parent: self,
                logs: kyoku,
            };

            ret.push(kyoku_log);
        }

        ret
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.logs.is_empty()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.logs.len()
    }
}

impl From<RawPartialLog<'_>> for RawLog {
    fn from(partial_log: RawPartialLog) -> Self {
        RawLog {
            logs: partial_log.logs.to_vec(),
            ..partial_log.parent.clone()
        }
    }
}

impl Log {
    /// Parse a tenhou.net/6 log from JSON string.
    #[inline]
    pub fn from_json_str(json_string: &str) -> Result<Self> {
        let raw_log: RawLog = json::from_str(json_string)?;
        Ok(Self::from(raw_log))
    }

    #[inline]
    pub fn filter_kyokus(&mut self, kyoku_filter: &KyokuFilter) {
        self.kyokus
            .retain(|l| kyoku_filter.test(l.meta.kyoku_num, l.meta.honba))
    }
}

impl From<RawLog> for Log {
    fn from(raw_log: RawLog) -> Self {
        let RawLog {
            logs, names, rule, ..
        } = raw_log;

        let game_length = if rule.disp.contains('東') {
            GameLength::Tonpuu
        } else {
            GameLength::Hanchan
        };
        let has_aka = rule.aka + rule.aka51 + rule.aka52 + rule.aka53 > 0;

        let kyokus = logs
            .into_iter()
            .map(|log| {
                let mut item = Kyoku {
                    meta: log.meta,
                    scoreboard: log.scoreboard,
                    dora_indicators: log.dora_indicators,
                    ura_indicators: log.ura_indicators,
                    action_tables: [
                        ActionTable {
                            haipai: log.haipai_0,
                            takes: log.takes_0,
                            discards: log.discards_0,
                        },
                        ActionTable {
                            haipai: log.haipai_1,
                            takes: log.takes_1,
                            discards: log.discards_1,
                        },
                        ActionTable {
                            haipai: log.haipai_2,
                            takes: log.takes_2,
                            discards: log.discards_2,
                        },
                        ActionTable {
                            haipai: log.haipai_3,
                            takes: log.takes_3,
                            discards: log.discards_3,
                        },
                    ],
                    end_status: kyoku::EndStatus::Ryukyoku {
                        score_deltas: [0; 4], // default
                    },
                };

                if let Some(json_scheme::ResultItem::Status(status_text)) = log.results.get(0) {
                    if status_text == "和了" {
                        let hora_details = log.results[1..]
                            .chunks_exact(2)
                            .filter_map(|detail_tuple| {
                                if let (
                                    json_scheme::ResultItem::ScoreDeltas(score_deltas),
                                    json_scheme::ResultItem::HoraDetail(who_target_tuple),
                                ) = (&detail_tuple[0], &detail_tuple[1])
                                {
                                    // TODO: it can actually fail, maybe impl TryFrom instead
                                    let hora_detail = kyoku::HoraDetail {
                                        score_deltas: *score_deltas,
                                        who: who_target_tuple[0].as_u64().unwrap_or(0) as u8,
                                        target: who_target_tuple[1].as_u64().unwrap_or(0) as u8,
                                    };
                                    Some(hora_detail)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        item.end_status = kyoku::EndStatus::Hora {
                            details: hora_details,
                        };
                    } else {
                        let score_deltas = if let Some(json_scheme::ResultItem::ScoreDeltas(dts)) =
                            log.results.get(1)
                        {
                            *dts
                        } else {
                            [0; 4]
                        };

                        item.end_status = kyoku::EndStatus::Ryukyoku { score_deltas };
                    }
                }

                item
            })
            .collect();

        Log {
            names,
            game_length,
            has_aka,
            kyokus,
        }
    }
}
