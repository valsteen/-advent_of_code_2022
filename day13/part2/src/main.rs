extern crate core;

use std::{cmp::Ordering, error::Error, io::stdin, io::BufRead, num::ParseIntError};

use nom::branch::alt;
use nom::character::complete::{char, digit1};
use nom::combinator::{cut, map, map_res};
use nom::error::{context, ContextError, FromExternalError, ParseError, VerboseError};
use nom::multi::separated_list0;
use nom::sequence::{preceded, terminated};
use nom::IResult;

#[derive(Debug, Clone)]
enum Expression {
    Array(Vec<Expression>),
    Item(u8),
}

trait ExpressionParseError<'a>:
    ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError>
{
}

impl<'a> ExpressionParseError<'a> for VerboseError<&'a str> {}

fn number<'a, E: ExpressionParseError<'a>>(i: &'a str) -> IResult<&'a str, u8, E> {
    map_res(digit1, |s: &str| s.parse::<u8>())(i)
}

fn expression<'a, E: ExpressionParseError<'a>>(i: &'a str) -> IResult<&'a str, Expression, E> {
    alt((map(array, Expression::Array), map(number, Expression::Item)))(i)
}

fn array<'a, E: ExpressionParseError<'a>>(i: &'a str) -> IResult<&'a str, Vec<Expression>, E> {
    context(
        "array",
        preceded(
            char('['),
            cut(terminated(
                separated_list0(char(','), expression),
                char(']'),
            )),
        ),
    )(i)
}

impl Eq for Expression {}

impl PartialEq<Self> for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl PartialOrd<Self> for Expression {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cmp(other).into()
    }
}

impl Ord for Expression {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Item(left), Self::Item(right)) => left.cmp(right),
            (Self::Item(left), Self::Array(_)) => Self::Array(vec![Self::Item(*left)]).cmp(other),
            (Self::Array(_), Self::Item(right)) => self.cmp(&Self::Array(vec![Self::Item(*right)])),
            (Self::Array(left), Self::Array(right)) => {
                for (left, right) in left.iter().zip(right) {
                    match left.cmp(right) {
                        Ordering::Less => return Ordering::Less,
                        Ordering::Greater => return Ordering::Greater,
                        Ordering::Equal => (),
                    }
                }
                left.len().cmp(&right.len())
            }
        }
    }
}

fn parse(input: &str) -> Result<Expression, Box<dyn Error>> {
    match expression::<VerboseError<_>>(input) {
        Ok((s, e)) => {
            if !s.is_empty() {
                Err(format!("trailing content: {}", s).into())
            } else {
                Ok(e)
            }
        }
        Err(e) => Err(format!("parsing error: {}", e).into()),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();

    let dividers = ["[[2]]", "[[6]]"]
        .into_iter()
        .map(parse)
        .collect::<Result<Vec<Expression>, _>>()?;

    let mut expressions = lines
        .flatten()
        .filter_map(|l| if l.is_empty() { None } else { Some(parse(&l)) })
        .collect::<Result<Vec<Expression>, Box<dyn Error>>>()?;

    expressions.extend(dividers.clone());
    expressions.sort();

    let mut indices = expressions
        .iter()
        .enumerate()
        .filter_map(|(n, expression)| dividers.contains(expression).then_some(n));

    let score = indices
        .next()
        .ok_or("expected to find back the 2 dividers")?
        + 1;
    let score = score
        * (1 + indices
            .next()
            .ok_or("expected to find back the 2 dividers")?);
    if indices.next().is_some() {
        Err("expected to find back only 2 dividers")?;
    }
    println!("{}", score);
    Ok(())
}
