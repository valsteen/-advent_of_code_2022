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

struct Execution {
    instruction: Instruction,
    remaining: usize,
}

impl Execution {
    fn new(instruction: Instruction) -> Self {
        Self {
            remaining: instruction.cycles(),
            instruction,
        }
    }

    fn execute(mut self, state: isize) -> (Option<Self>, isize) {
        match &self.instruction {
            Instruction::Noop => (None, state),
            Instruction::AddX(add) => {
                self.remaining -= 1;
                if self.remaining == 0 {
                    (None, state + add.amount)
                } else {
                    (Some(self), state)
                }
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines().flatten();

    let compute = (0..).scan(
        (lines.map(|line| line.parse::<Instruction>()), None, 0),
        |(instructions, execution, x), pos| {
            let result = if (*x..=*x + 2).contains(&(pos % 40_isize)) {
                b'#'
            } else {
                b' '
            };

            (*execution, *x) = if let Some(execution) = execution.take() {
                execution
            } else {
                match instructions.next() {
                    Some(Ok(instruction)) => Execution::new(instruction),
                    Some(Err(e)) => return Some(Err(e)),
                    None => return None,
                }
            }
            .execute(*x);

            Some(Ok(result))
        },
    );

    for (pos, iteration) in compute.enumerate() {
        if pos % 40 == 0 {
            println!()
        }
        print!("{}", char::from(iteration?));
    }
    Ok(())
}
