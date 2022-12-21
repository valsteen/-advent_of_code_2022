use std::collections::{HashMap, VecDeque};

use std::{
    error::Error,
    fmt::Debug,
    io::{stdin, BufRead},
    num::ParseIntError,
};

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1, space1},
    combinator::{complete, map_res},
    error::{ContextError, FromExternalError, ParseError, VerboseError},
    sequence::tuple,
    Finish, IResult,
};

trait ExpressionParseError<'a>:
    ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + Debug
{
}

impl<'a> ExpressionParseError<'a> for VerboseError<&'a str> {}

fn number<'a, E: ExpressionParseError<'a>>(i: &'a str) -> IResult<&'a str, Expression, E> {
    let (rest, digits) = map_res(digit1, |s: &str| s.parse())(i)?;
    Ok((rest, Expression::Number(digits)))
}

fn expression<'a, E: ExpressionParseError<'a>>(i: &'a str) -> IResult<&'a str, Expression, E> {
    let (rest, (a, _, op, _, b)) = tuple((
        alphanumeric1,
        space1,
        alt((tag("+"), tag("-"), tag("/"), tag("*"))),
        space1,
        alphanumeric1,
    ))(i)?;

    let (a, b) = (a.to_string(), b.to_string());

    let operation = match op {
        "+" => Operation::Add,
        "-" => Operation::Subtract,
        "*" => Operation::Multiply,
        "/" => Operation::Divide,
        _ => unreachable!(),
    };

    Ok((rest, Expression::Operation(operation, a, b)))
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum Operation {
    Add = b'+',
    Subtract = b'-',
    Multiply = b'*',
    Divide = b'/',
}

#[derive(Debug)]
enum Expression {
    Operation(Operation, String, String),
    Number(i64),
}

#[derive(Debug)]
struct Monkey {
    name: String,
    expression: Expression,
}

fn monkey<'a, E: ExpressionParseError<'a>>(i: &'a str) -> IResult<&'a str, Monkey, E> {
    let (rest, (name, _, exp)) =
        complete(tuple((alphanumeric1, tag(": "), alt((number, expression)))))(i)?;

    Ok((
        rest,
        Monkey {
            name: name.to_string(),
            expression: exp,
        },
    ))
}

trait Solver {
    fn solve(&self, name: &str, cache: &mut HashMap<String, i64>) -> Result<i64, &'static str>;
}

fn solve(op: Operation, a: i64, b: i64) -> i64 {
    match op {
        Operation::Add => a + b,
        Operation::Subtract => a - b,
        Operation::Multiply => a * b,
        Operation::Divide => a / b,
    }
}

impl Solver for HashMap<String, Monkey> {
    fn solve(&self, name: &str, cache: &mut HashMap<String, i64>) -> Result<i64, &'static str> {
        if let Some(result) = cache.get(name) {
            return Ok(*result);
        }

        let expression = &self.get(name).ok_or("monkey not found")?.expression;

        let solved = match expression {
            Expression::Operation(op, a, b) => {
                let a = self.solve(a, cache)?;
                let b = self.solve(b, cache)?;
                solve(*op, a, b)
            }
            Expression::Number(n) => {
                cache.insert(name.to_string(), *n);
                *n
            }
        };
        cache.insert(name.to_string(), solved);
        Ok(solved)
    }
}

fn reorderings(
    a: String,
    op: Operation,
    b: String,
    c: String,
) -> [(String, (Operation, String, String)); 3] {
    match op {
        Operation::Add => [
            (a.clone(), (Operation::Add, b.clone(), c.clone())),
            (b.clone(), (Operation::Subtract, a.clone(), c.clone())),
            (c, (Operation::Subtract, a, b)),
        ],
        Operation::Subtract => [
            (a.clone(), (Operation::Subtract, b.clone(), c.clone())),
            (b.clone(), (Operation::Add, a.clone(), c.clone())),
            (c, (Operation::Subtract, b, a)),
        ],
        Operation::Multiply => [
            (a.clone(), (Operation::Multiply, b.clone(), c.clone())),
            (b.clone(), (Operation::Divide, a.clone(), c.clone())),
            (c, (Operation::Divide, a, b)),
        ],
        Operation::Divide => [
            (a.clone(), (Operation::Divide, b.clone(), c.clone())),
            (b.clone(), (Operation::Multiply, a.clone(), c.clone())),
            (c, (Operation::Divide, b, a)),
        ],
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut monkeys = {
        let lines = stdin().lock().lines();
        lines
            .map(|line| {
                let line = line.map_err(Box::new)?;
                let instruction = monkey::<VerboseError<_>>(&line)
                    .finish()
                    .map(|(_, line)| line)
                    .map_err(|err| err.to_string())?;
                Ok((instruction.name.clone(), instruction.expression))
            })
            .collect::<Result<HashMap<String, Expression>, Box<dyn Error>>>()?
    };

    let root = monkeys.remove("root").ok_or("root monkey not found")?;

    let Expression::Operation(_, root_a, root_b) = root else {
        return Err("root monkey should be an operation".into())
    };

    monkeys.remove("humn").ok_or("human not found")?;

    let mut known = HashMap::new();

    let mut expressions = monkeys
        .into_iter()
        .filter_map(|(a, expression)| match expression {
            Expression::Operation(op, b, c) => {
                let a = if a == root_a || a == root_b {
                    "root".to_string()
                } else {
                    a
                };
                let b = if b == root_a || b == root_b {
                    "root".to_string()
                } else {
                    b
                };
                let c = if c == root_a || c == root_b {
                    "root".to_string()
                } else {
                    c
                };

                Some(reorderings(a, op, b, c))
            }
            Expression::Number(i) => {
                known.insert(a, i);
                None
            }
        })
        .flatten()
        .collect::<VecDeque<_>>();

    while let Some((a, (operation, b, c))) = expressions.pop_front() {
        if let (Some(i), Some(j)) = (known.get(&b), known.get(&c)) {
            known.insert(a, solve(operation, *i, *j));
            continue;
        };
        expressions.push_back((a, (operation, b, c)))
    }
    println!("{}", known["humn"]);
    Ok(())
}
