use core::fmt;

use rand::{Rng, seq::SliceRandom, thread_rng};
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
                let letter = (rand::random::<u8>() % 26 + b'a') as char;

                let client = Client::new();
                let response = client
                    .get(&format!(
                        "https://api.datamuse.com/words?sp={0}*&&md=p",
                        letter
                    ))
                    .send();

                match response {
                    Ok(res) => match res.json::<Vec<Value>>() {
                        Ok(words) => {
                            let adjectives: Vec<String> = words
                                .iter()
                                .filter_map(|word| {
                                    let w = word["word"].as_str()?;
                                    let tags = word["tags"].as_array()?;
                                    (w.len() >= 4 && w.len() <= 8 && w.starts_with(letter) && tags.contains(&Value::String("adj".to_string())))
                                        .then(|| w.to_owned())
                                })
                                .collect();

                            let nouns: Vec<String> = words
                                .iter()
                                .filter_map(|word| {
                                    let w = word["word"].as_str()?;
                                    let tags = word["tags"].as_array()?;
                                    (w.len() >= 4 && w.len() <= 8 && w.starts_with(letter) && tags.contains(&Value::String("n".to_string())))
                                        .then(|| w.to_owned())
                                })
                                .collect();

                            if let (Some(adj), Some(noun)) = (
                                adjectives.choose(&mut rand::thread_rng()).cloned(),
                                nouns.choose(&mut rand::thread_rng()).cloned(),
                            ) {
                                *self = Identifier::Success {
                                    uuid: *uuid,
                                    name: format!("{} {}", adj, noun),
                                };
                            } else {
                                *self = Identifier::Failure {
                                    uuid: *uuid,
                                    error: "Error fetching word.".to_string(),
                                };
                            }
                        }
                        Err(_) => {
                            *self = Identifier::Failure {
                                uuid: *uuid,
                                error: "Error parsing API response.".to_string(),
                            };
                        }
                    },
                    Err(_) => {
                        *self = Identifier::Failure {
                            uuid: *uuid,
                            error: "Error making API request.".to_string(),
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
