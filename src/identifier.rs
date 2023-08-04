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

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam::channel;
    use std::collections::HashMap;
    use std::io::{Write, stdout};
    use std::sync::{Arc, atomic::{AtomicUsize, AtomicBool, Ordering}};
    use std::thread;
    use std::time::{Instant, Duration};

    #[test]
    fn test_uniqueness() {
        let num_cpus = num_cpus::get(); // get the number of logical cores
        let pool = rayon::ThreadPoolBuilder::new().num_threads(num_cpus).build().unwrap(); // create a ThreadPool with that many threads

        let (tx, rx) = channel::unbounded();
        let total = 10000;
        let start = Arc::new(Instant::now());
        let counter = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));

        let counter_clone = Arc::clone(&counter);
        let start_clone = Arc::clone(&start);
        let stop_clone = Arc::clone(&stop);
        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(1));
                let elapsed = start_clone.elapsed();
                let count = counter_clone.load(Ordering::SeqCst);
                print!("\rElapsed time: {:0>2}:{:0>2}, Identifiers created: {}/{}",
                    elapsed.as_secs() / 60, 
                    elapsed.as_secs() % 60, 
                    count, 
                    total
                );
                stdout().flush().unwrap();
                if count >= total {
                    stop_clone.store(true, Ordering::SeqCst);
                    break;
                } else if elapsed >= Duration::from_secs(60 * 60) {
                    panic!("Test took too long.");
                }
            }
        });

        let stop_clone = Arc::clone(&stop);
        // using the thread pool to generate identifiers in parallel
        pool.scope(|s| {
            for _ in 0..total {
                if stop_clone.load(Ordering::SeqCst) {
                    return;
                }
                let tx_clone = tx.clone();
                let counter_clone = Arc::clone(&counter);  // clone the counter before moving it into the closure
                s.spawn(move |_| {
                    let mut identifier = Identifier::new();
                    identifier.set();
                    tx_clone.send(identifier).unwrap();
                    counter_clone.fetch_add(1, Ordering::SeqCst);
                });
            }
        });
        
        drop(tx);  // close the channel

        let mut word_counts: HashMap<String, usize> = HashMap::new();
        let mut pair_counts: HashMap<String, usize> = HashMap::new();

        for identifier in rx {
            if let Identifier::Success { name, .. } = identifier {
                let words: Vec<&str> = name.split_whitespace().collect();
                if words.len() == 2 {
                    *word_counts.entry(words[0].to_string()).or_insert(0) += 1;
                    *word_counts.entry(words[1].to_string()).or_insert(0) += 1;
                    *pair_counts.entry(name).or_insert(0) += 1;
                }
            }
        }

        println!("\n\nRepeated words:");
        for (word, count) in word_counts.iter().filter(|(_, count)| **count > 1) {
            println!("{}: {}", word, count);
        }

        println!("\n\nRepeated pairs:");
        for (pair, count) in pair_counts.iter().filter(|(_, count)| **count > 1) {
            println!("{}: {}", pair, count);
        }
    }
}

