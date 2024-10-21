pub mod algorithms;

pub fn play<G: Guesser>(answer: &'static str, mut guesser: G) -> Option<usize> {
    // play six rounds where it invokes guesser each round
    let mut history = Vec::new();
    // while wordle only allows for six guesses, we will limit
    // our guesses so we do not cause stack overflow
    for i in 1..=32 {
        let guess = guesser.guess(&history);
        if guess == answer {
            return Some(i);
        }
        let correctness = Correctness::compute(answer, &guess);
        history.push(Guess {
            word: guess,
            mask: correctness,
        });
    }
    None
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
        }
        c
    }
}

pub struct Guess {
    pub word: String,
    pub mask: [Correctness; 5],
}

pub trait Guesser {
    // function that makes a guess; takes info of current guess progress as as arguments
    fn guess(&mut self, history: &[Guess]) -> String;
}

#[cfg(test)]
mod tests {
    mod compute {
        use crate::Correctness;
        #[test]
        fn basic() {
            assert_eq!(
                Correctness::compute("abcde", "abcde"),
                [Correctness::Correct; 5]
            )
        }
    }
}
