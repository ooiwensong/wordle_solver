use std::collections::HashMap;

use crate::{Guess, Guesser};

const DICTIONARY: &str = include_str!("../../dictionary.txt");

pub struct Naive {
    remaining: HashMap<&'static str, usize>,
}

#[derive(Debug, Clone, Copy)]
struct Candidate {
    word: &'static str,
    count: usize,
    goodness: f64,
}

impl Naive {
    pub fn new() -> Self {
        Self {
            remaining: HashMap::from_iter(DICTIONARY.lines().map(|line| {
                let (word, count) = line
                    .split_once(' ')
                    .expect("every line is word + space + word count");
                let count: usize = count.parse().expect("every count is a number");
                (word, count)
            })),
        }
    }
}

impl Guesser for Naive {
    fn guess(&mut self, history: &[Guess]) -> String {
        if let Some(last) = history.last() {
            self.remaining.retain(|word, _| last.matches(word));
        }
        let mut best: Option<Candidate> = None;
        let goodness = 0.0;
        for (&word, &count) in &self.remaining {
            if let Some(c) = best {
                // todo!();
                if goodness > c.goodness {
                    best = Some(Candidate {
                        word,
                        count,
                        goodness,
                    })
                }
            } else {
                best = Some(Candidate {
                    word,
                    count,
                    goodness,
                });
            }
        }
        best.unwrap().word.to_string()
    }
}
