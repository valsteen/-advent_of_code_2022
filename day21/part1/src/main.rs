use std::collections::HashMap;
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
#[derive(Debug)]
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

                match op {
                    Operation::Add => a + b,
                    Operation::Subtract => a - b,
                    Operation::Multiply => a * b,
                    Operation::Divide => a / b,
                }
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

fn main() -> Result<(), Box<dyn Error>> {
    let monkeys = {
        let lines = stdin().lock().lines();
        lines
            .map(|line| {
                let line = line.map_err(Box::new)?;
                let instruction = monkey::<VerboseError<_>>(&line)
                    .finish()
                    .map(|(_, line)| line)
                    .map_err(|err| err.to_string())?;
                Ok((instruction.name.clone(), instruction))
            })
            .collect::<Result<HashMap<String, Monkey>, Box<dyn Error>>>()?
    };

    let value = monkeys.solve("root", &mut Default::default())?;
    println!("{}", value);
    Ok(())
}
