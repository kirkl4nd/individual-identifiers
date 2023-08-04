use core::fmt;
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use reqwest::blocking::Client;
use serde_json::Value;
use std::sync::Mutex;
use uuid::Uuid;

const ERROR_FETCHING_WORD: &str = "Error fetching word.";
const ERROR_PARSING_API: &str = "Error parsing API response.";
const ERROR_API_REQUEST: &str = "Error making API request.";

lazy_static! {
    static ref CLIENT: Mutex<Client> = Mutex::new(Client::new());
}

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
                let url = format!("https://api.datamuse.com/words?sp={0}*&&md=p", letter);
                let client = CLIENT.lock().unwrap();
                let response = client.get(&url).send();

                match response {
                    Ok(res) => match res.json::<Vec<Value>>() {
                        Ok(words) => {
                            let adjectives = filter_words(&words, &letter, "adj");
                            let nouns = filter_words(&words, &letter, "n");

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
                                    error: ERROR_FETCHING_WORD.to_string(),
                                };
                            }
                        }
                        Err(_) => {
                            *self = Identifier::Failure {
                                uuid: *uuid,
                                error: ERROR_PARSING_API.to_string(),
                            };
                        }
                    },
                    Err(_) => {
                        *self = Identifier::Failure {
                            uuid: *uuid,
                            error: ERROR_API_REQUEST.to_string(),
                        };
                    }
                }
            }
        }
    }
}

fn filter_words(words: &[Value], letter: &char, tag: &str) -> Vec<String> {
    words
        .iter()
        .filter_map(|word| {
            let w = word["word"].as_str()?;
            let tags = word["tags"].as_array()?;
            (w.len() >= 4
                && w.len() <= 8
                && w.starts_with(*letter)
                && tags.contains(&Value::String(tag.to_string())))
            .then(|| w.to_owned())
        })
        .collect()
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
