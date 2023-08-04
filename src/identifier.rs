use core::fmt;

use rand::Rng;
use reqwest::blocking::Client;
use serde_json::Value;
use uuid::Uuid;

pub enum Identifier {
    Default { uuid: Uuid },
    Success { uuid: Uuid, name: String },
    Failure { uuid: Uuid, error: String },
}

impl Identifier {
    pub fn new() -> Self {
        Self::Default {
            uuid: Uuid::new_v4(),
        }
    }

    pub fn set(&mut self) {
        match self {
            Identifier::Success { .. } => {}
            Identifier::Default { uuid } | Identifier::Failure { uuid, .. } => {
                let letter = (rand::thread_rng().gen::<u8>() % 26 + b'a') as char;

                let client = Client::new();
                let get_word = |kind: &str| {
                    client
                        .get(&format!(
                            "https://api.datamuse.com/words?sp={0}*&rel_jj{1}={0}",
                            letter, kind
                        ))
                        .send()
                        .ok()
                        .and_then(|response| response.json::<Vec<Value>>().ok())
                        .and_then(|words| {
                            words
                                .into_iter()
                                .find(|word| {
                                    let w = word["word"].as_str().unwrap_or("");
                                    w.len() <= 7 && w.starts_with(letter)
                                })
                                .map(|word| word["word"].as_str().unwrap_or("").to_owned())
                        })
                };

                match (get_word("b"), get_word("a")) {
                    (Some(adj), Some(noun)) => {
                        *self = Identifier::Success {
                            uuid: *uuid,
                            name: format!("{} {}", adj.to_uppercase(), noun),
                        };
                    }
                    (err_adj, err_noun) => {
                        let error_message =
                            format!("Adjective error: {:?}, Noun error: {:?}", err_adj, err_noun);
                        *self = Identifier::Failure {
                            uuid: *uuid,
                            error: error_message,
                        };
                    }
                }
            }
        }
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Identifier::Default { uuid } => {
                write!(f, "{}", uuid)
            }

            Identifier::Success { uuid, name } => {
                write!(f, "{}\t({})", uuid, name)
            }

            Identifier::Failure { uuid, error } => {
                write!(f, "{}\tError fetching name:\t{}", uuid, error)
            }
        }
    }
}
