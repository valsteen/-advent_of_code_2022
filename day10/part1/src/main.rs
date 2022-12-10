use recap::Recap;
use serde::Deserialize;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::str::FromStr;

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^addx (?P<amount>-?[0-9]+)$"#)]
struct AddX {
    amount: isize,
}

#[derive(Debug)]
enum Instruction {
    Noop,
    AddX(AddX),
}

impl FromStr for Instruction {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self::AddX).or_else(|_| {
            Ok((s == "noop")
                .then_some(Self::Noop)
                .ok_or("invalid instruction")?)
        })
    }
}

impl Instruction {
    fn cycles(&self) -> usize {
        match self {
            Instruction::Noop => 1,
            Instruction::AddX(_) => 2,
        }
    }
}

struct State {
    x: isize,
    cycles: usize,
}

impl Default for State {
    fn default() -> Self {
        Self { x: 1, cycles: 1 }
    }
}

impl State {
    fn execute(&mut self, i: Instruction) {
        match i {
            Instruction::Noop => {}
            Instruction::AddX(x) => self.x += x.amount,
        }
    }

    fn strength_for(x: isize, cycles: usize) -> isize {
        cycles as isize * x
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let mut next_inspection = 20;
    let (result, _) =
        lines
            .flatten()
            .try_fold((0, State::default()), |(mut sum, mut state), line| {
                let instruction: Instruction = line.parse()?;
                state.cycles += instruction.cycles();
                if state.cycles > next_inspection {
                    sum += State::strength_for(state.x, next_inspection);
                    next_inspection += 40;
                }
                state.execute(instruction);
                if state.cycles == next_inspection {
                    sum += State::strength_for(state.x, next_inspection);
                    next_inspection += 40;
                }
                Ok::<_, Box<dyn Error>>((sum, state))
            })?;

    println!("{}", result);
    Ok(())
}
