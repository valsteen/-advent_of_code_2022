use recap::Recap;
use serde::Deserialize;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::ops::ControlFlow;
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

struct State<T: Iterator<Item = Result<Instruction, Box<dyn Error>>>> {
    next_inspection: usize,
    instructions: T,
    current_execution: Option<Execution>,
    sum: isize,
    x: isize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines().flatten();

    let result = (2..).try_fold(
        State {
            next_inspection: 20,
            instructions: lines.map(|line| line.parse::<Instruction>()),
            current_execution: None,
            sum: 0,
            x: 1,
        },
        |mut state, cycles| {
            let (execution, next) = if let Some(execution) = state.current_execution {
                execution
            } else {
                match state.instructions.next() {
                    Some(Ok(instruction)) => Execution::new(instruction),
                    Some(Err(e)) => return ControlFlow::Break(Err(e)),
                    None => return ControlFlow::Break(Ok(state.sum)),
                }
            }
            .execute(state.x);
            state.x = next;
            state.current_execution = execution;

            if cycles == state.next_inspection {
                state.next_inspection += 40;
                state.sum += cycles as isize * state.x;
            }

            ControlFlow::Continue(state)
        },
    );

    match result {
        ControlFlow::Continue(_) => Err("Program unexpectedly halted")?,
        ControlFlow::Break(Ok(sum)) => {
            println!("{}", sum);
            Ok(())
        }
        ControlFlow::Break(Err(err)) => Err(err)?,
    }
}
