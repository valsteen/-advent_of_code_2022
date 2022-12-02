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

    fn play(&self, other: Hand) -> Outcome {
        match (self, other) {
            (Hand::Rock, Hand::Rock) => Outcome::Draw,
            (Hand::Rock, Hand::Paper) => Outcome::Lost,
            (Hand::Rock, Hand::Scissors) => Outcome::Won,
            (Hand::Paper, Hand::Rock) => Outcome::Won,
            (Hand::Paper, Hand::Paper) => Outcome::Draw,
            (Hand::Paper, Hand::Scissors) => Outcome::Lost,
            (Hand::Scissors, Hand::Rock) => Outcome::Lost,
            (Hand::Scissors, Hand::Paper) => Outcome::Won,
            (Hand::Scissors, Hand::Scissors) => Outcome::Draw,
        }
    }
}

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
    me: Hand,
    them: Hand,
}

impl Round {
    fn score(&self) -> usize {
        self.me.score() + self.me.play(self.them).score()
    }

    fn new(me: Hand, them: Hand) -> Self {
        Self { me, them }
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
        let me = match chars.next().ok_or("expected character")? {
            'X' => Hand::Rock,
            'Y' => Hand::Paper,
            'Z' => Hand::Scissors,
            _ => Err("Unexpected hand")?,
        };
        Ok(Round { me, them })
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
