use recap::Recap;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use std::error::Error;
use std::io::{stdin, BufRead};
use std::str::FromStr;

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^Monkey (?P<n>[0-9]+):$"#)]
struct MonkeyNumber {
    n: usize,
}

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^  Starting items: ((?P<level>[0-9]+)(, )?)+$"#)]
struct Items {
    level: Vec<usize>,
}

#[derive(Debug)]
enum Operator {
    Plus,
    Multiply,
}

impl FromStr for Operator {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut bytes = s.bytes();
        let char = bytes.next().ok_or("operand should have one character")?;
        if bytes.next().is_some() {
            return Err("operator should be just one character");
        }

        match char {
            b'+' => Ok(Self::Plus),
            b'*' => Ok(Self::Multiply),
            _ => Err("invalid operator"),
        }
    }
}

#[serde_as]
#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^  Operation: new = old (?P<operator>.) (?P<operand>.+)$"#)]
struct Operation {
    #[serde_as(as = "DisplayFromStr")]
    operator: Operator,
    #[serde_as(as = "DisplayFromStr")]
    operand: Operand,
}

#[derive(Debug)]
enum Operand {
    Number(usize),
    Old,
}

impl FromStr for Operand {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(n) = s.parse() {
            Ok(Self::Number(n))
        } else if s == "old" {
            Ok(Self::Old)
        } else {
            Err("invalid operand")
        }
    }
}

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^  Test: divisible by (?P<divisible>[0-9]+)$"#)]
struct Test {
    divisible: usize,
}

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^    If (?P<boolean>.+): throw to monkey (?P<monkey>[0-9]+)$"#)]
struct Condition {
    boolean: bool,
    monkey: usize,
}

#[derive(Debug)]
struct Monkey {
    items: Vec<usize>,
    operation: Operation,
    divisible: usize,
    if_true: usize,
    if_false: usize,
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let lines = lines.flatten().filter(|line| !line.is_empty());
    let monkeys = (0..).scan(lines, |lines, n| {
        match MonkeyNumber::from_str(&lines.next()?) {
            Ok(monkey) => {
                if monkey.n != n {
                    return Some(Err("unexpected monkey number".into()));
                }
            }
            Err(e) => return Some(Err(e.into())),
        }
        let mut parse_monkey = || {
            let items = Items::from_str(&lines.next().ok_or("expected items")?)?.level;
            let operation = Operation::from_str(&lines.next().ok_or("expected operation")?)?;
            let divisible = Test::from_str(&lines.next().ok_or("expected test")?)?.divisible;
            let condition_true = Condition::from_str(&lines.next().ok_or("expected test")?)?;
            if !condition_true.boolean {
                Err("expected true condition")?
            }
            let condition_false = Condition::from_str(&lines.next().ok_or("expected test")?)?;
            if condition_false.boolean {
                Err("expected false condition")?
            }
            Ok(Monkey {
                items,
                operation,
                divisible,
                if_true: condition_true.monkey,
                if_false: condition_false.monkey,
            })
        };

        Some(parse_monkey())
    }).collect::<Result<Vec<Monkey>, Box<dyn Error>>>()?;

    for monkey in monkeys {
        println!("{:?}", monkey);
    }
    Ok(())
}
