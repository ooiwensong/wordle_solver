use std::collections::HashSet;

pub mod algorithms;

const DICTIONARY: &str = include_str!("../dictionary.txt");

pub struct Wordle {
    dictionary: HashSet<&'static str>,
}

impl Wordle {
    pub fn new() -> Self {
        Self {
            // we want every other element because we want to omit the word count
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("every word is a word + space + word count")
                    .0
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {
        // play six rounds where it invokes guesser each round
        let mut history = Vec::new();
        // while wordle only allows for six guesses, we will limit
        // our guesses so we do not cause stack overflow
        for i in 1..=32 {
            let guess = guesser.guess(&history);
            if guess == answer {
                return Some(i);
            }

            // not sure why we need to deref and ref 'guess' again
            assert!(self.dictionary.contains(&*guess));

            let correctness = Correctness::compute(answer, &guess);
            history.push(Guess {
                word: guess,
                mask: correctness,
            });
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Correctness {
    // Green
    Correct,
    // Yellow,
    Misplaced,
    // Gray,
    Wrong,
}

impl Correctness {
    fn compute(answer: &str, guess: &str) -> [Self; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        // initialise c as an array of five Wrong guesses
        let mut c = [Correctness::Wrong; 5];

        // Mark guesses correct
        for (i, (a, g)) in answer.chars().zip(guess.chars()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
            }
        }
        // Mark guesses misplaced
        let mut used = [false; 5];
        for (i, &c) in c.iter().enumerate() {
            if c == Correctness::Correct {
                used[i] = true;
            }
        }
        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                continue; // already marked as correct
            }
            // if the current guess letter matches any letters inside the answer
            // true => mark as Misplaced
            // false => remains Wrong
            if answer.chars().enumerate().any(|(i, a)| {
                if a == g && !used[i] {
                    used[i] = true;
                    return true;
                }
                false
            }) {
                c[i] = Correctness::Misplaced
            }
        }
        c
    }
}

pub struct Guess {
    pub word: String,
    pub mask: [Correctness; 5],
}

impl Guess {
    pub fn matches(&self, word: &str) -> bool {
        assert_eq!(self.word.len(), 5);
        assert_eq!(word.len(), 5);

        // first check greens
        let mut used = [false; 5];
        for (i, ((g, &m), w)) in self
            .word
            .chars()
            .zip(&self.mask)
            .zip(word.chars())
            .enumerate()
        {
            if m == Correctness::Correct {
                if g != w {
                    return false;
                } else {
                    used[i] = true;
                    continue;
                }
            }
        }
        for (_, ((_, _), w)) in self
            .word
            .chars()
            .zip(&self.mask)
            .zip(word.chars())
            .enumerate()
        {
            let mut plausible = true;
            if self
                .word
                .chars()
                .zip(&self.mask)
                .enumerate()
                .any(|(j, (g, m))| {
                    if g != w {
                        return false;
                    }
                    if used[j] {
                        return false;
                    }
                    // we are looking at a 'w' in 'word' and have found a 'w' in the previous guess.
                    // the colour of that previous 'w' will tell us whether this 'w' might be ok.
                    match m {
                        Correctness::Correct => unreachable!(
                            "all correct guesses should have resulted in return or be used"
                        ),
                        Correctness::Misplaced if j == 1 => {
                            // 'w'
                            used[j] = true;
                            return true;
                        }
                        Correctness::Misplaced => {
                            used[j] = true;
                            return true;
                        }
                        Correctness::Wrong => {
                            plausible = false;
                            return false;
                        }
                    }
                })
                && plausible
            {
                // the char 'w' was yellow in the previous guess
            } else if !plausible {
                return false;
            } else {
                // we have no info about char 'w', so word might still match
            }
        }
        true
    }
}

pub trait Guesser {
    // function that makes a guess; takes info of current guess progress as as arguments
    fn guess(&mut self, history: &[Guess]) -> String;
}

impl Guesser for fn(history: &[Guess]) -> String {
    fn guess(&mut self, history: &[Guess]) -> String {
        (*self)(history)
    }
}

#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl $crate::Guesser for G {
            fn guess(&mut self, $history: &[Guess]) -> String {
                $impl
            }
        }
        G
    }};
}

#[cfg(test)]
macro_rules! mask {
    (C) => {$crate::Correctness::Correct};
    (M) => {$crate::Correctness::Misplaced};
    (W) => {$crate::Correctness::Wrong};
    ($($c:tt)+) => {[
        $(mask!($c)),+
    ]}
}

#[cfg(test)]
mod tests {
    mod guess_matcher {
        use crate::Guess;

        #[test]
        fn matches() {
            assert!(Guess {
                word: "abcde".to_string(),
                mask: mask![C C C C C],
            }
            .matches("abcde"));
        }
    }
    mod game {
        use crate::{Guess, Wordle};

        // make sure the code is playing the game correctly
        #[test]
        fn genius() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "right".to_string() });
            assert_eq!(w.play("right", guesser), Some(1));
        }

        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(2));
        }

        #[test]
        fn impressive() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(3));
        }

        #[test]
        fn splendid() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(4));
        }

        #[test]
        fn great() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(5));
        }

        #[test]
        fn phew() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return "right".to_string();
                }
                "wrong".to_string()
            });
            assert_eq!(w.play("right", guesser), Some(6));
        }

        #[test]
        fn ooops() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "wrong".to_string() });
            assert_eq!(w.play("right", guesser), None);
        }
    }

    mod compute {
        use crate::Correctness;
        #[test]
        fn all_correct() {
            assert_eq!(
                Correctness::compute("abcde", "abcde"),
                [Correctness::Correct; 5]
            )
        }

        #[test]
        fn all_wrong() {
            assert_eq!(
                Correctness::compute("abcde", "ghjkl"),
                [Correctness::Wrong; 5]
            )
        }
        #[test]
        fn all_misplaced() {
            assert_eq!(
                Correctness::compute("abcde", "eabcd"),
                [Correctness::Misplaced; 5]
            )
        }

        #[test]
        fn repeat_green() {
            assert_eq!(
                Correctness::compute("aabbb", "aaccc"),
                [
                    Correctness::Correct,
                    Correctness::Correct,
                    Correctness::Wrong,
                    Correctness::Wrong,
                    Correctness::Wrong,
                ]
            )
        }

        #[test]
        fn repeat_yellow() {
            assert_eq!(
                Correctness::compute("aabbb", "ccaac"),
                [
                    Correctness::Wrong,
                    Correctness::Wrong,
                    Correctness::Misplaced,
                    Correctness::Misplaced,
                    Correctness::Wrong,
                ]
            )
        }

        #[test]
        fn repeat_some_green() {
            assert_eq!(
                Correctness::compute("aabbb", "caacc"),
                [
                    Correctness::Wrong,
                    Correctness::Correct,
                    Correctness::Misplaced,
                    Correctness::Wrong,
                    Correctness::Wrong,
                ]
            )
        }

        #[test]
        fn correct_number_of_misplaced() {
            assert_eq!(
                Correctness::compute("azzaz", "aaabb"),
                [
                    Correctness::Correct,
                    Correctness::Misplaced,
                    Correctness::Wrong,
                    Correctness::Wrong,
                    Correctness::Wrong,
                ]
            )
        }

        #[test]
        fn correct_number_of_correct() {
            assert_eq!(
                Correctness::compute("baccc", "aaddd"),
                [
                    Correctness::Wrong,
                    Correctness::Correct,
                    Correctness::Wrong,
                    Correctness::Wrong,
                    Correctness::Wrong,
                ]
            )
        }

        #[test]
        fn correct_number_of_correct2() {
            assert_eq!(
                Correctness::compute("abcde", "aacde"),
                [
                    Correctness::Correct,
                    Correctness::Wrong,
                    Correctness::Correct,
                    Correctness::Correct,
                    Correctness::Correct,
                ]
            )
        }
    }
}
