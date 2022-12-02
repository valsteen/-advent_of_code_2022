use std::error::Error;
use std::io::{stdin, BufRead};
use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
enum Hand {
    Rock,
    Paper,
    Scissors,
}

impl Hand {
    fn score(&self) -> usize {
        match self {
            Hand::Rock => 1,
            Hand::Paper => 2,
            Hand::Scissors => 3,
        }
    }

    fn new(them: Self, outcome: Outcome) -> Self {
        match (them, outcome) {
            (Hand::Rock, Outcome::Won) => Hand::Paper,
            (Hand::Rock, Outcome::Draw) => Hand::Rock,
            (Hand::Rock, Outcome::Lost) => Hand::Scissors,

            (Hand::Paper, Outcome::Won) => Hand::Scissors,
            (Hand::Paper, Outcome::Draw) => Hand::Paper,
            (Hand::Paper, Outcome::Lost) => Hand::Rock,

            (Hand::Scissors, Outcome::Won) => Hand::Rock,
            (Hand::Scissors, Outcome::Draw) => Hand::Scissors,
            (Hand::Scissors, Outcome::Lost) => Hand::Paper,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Outcome {
    Lost,
    Draw,
    Won,
}

impl Outcome {
    fn score(&self) -> usize {
        match self {
            Outcome::Lost => 0,
            Outcome::Draw => 3,
            Outcome::Won => 6,
        }
    }
}

#[derive(Debug)]
struct Round {
    them: Hand,
    outcome: Outcome,
}

impl Round {
    fn score(&self) -> usize {
        Hand::new(self.them, self.outcome).score() + self.outcome.score()
    }
}

impl FromStr for Round {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 3 {
            Err("unexpected line")?;
        }
        let mut chars = s.chars();
        let them = match chars.next().ok_or("expected character")? {
            'A' => Hand::Rock,
            'B' => Hand::Paper,
            'C' => Hand::Scissors,
            _ => Err("Unexpected hand")?,
        };
        if chars.next().ok_or("expected character")? != ' ' {
            Err("expected space")?
        }
        let outcome = match chars.next().ok_or("expected character")? {
            'X' => Outcome::Lost,
            'Y' => Outcome::Draw,
            'Z' => Outcome::Won,
            _ => Err("Unexpected hand")?,
        };
        Ok(Round { outcome, them })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let score = lines.flatten().try_fold(0, |acc, line| {
        Ok::<_, Box<dyn Error>>(acc + line.parse::<Round>()?.score())
    })?;

    println!("{}", score);
    Ok(())
}
