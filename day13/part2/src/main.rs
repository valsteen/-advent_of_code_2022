use std::cmp::Ordering;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::str::FromStr;

#[derive(Debug, Clone)]
enum Expression {
    List(Vec<Expression>),
    Item(usize),
}

impl Expression {
    fn parse(mut input: &[u8]) -> Result<(Expression, &[u8]), &'static str> {
        if input.is_empty() {
            return Err("empty string");
        }

        match input[0] {
            b'[' => {
                let mut result = Vec::new();
                loop {
                    if input[1] == b']' {
                        return Ok((Expression::List(Vec::new()), &input[2..]));
                    }
                    let (sub, rest) = Self::parse(&input[1..])?;
                    input = rest;
                    if input.is_empty() {
                        Err("Unexpected end of expression")?
                    }
                    result.push(sub);
                    match input[0] {
                        b',' => {
                            continue;
                        }
                        b']' => return Ok((Expression::List(result), &input[1..])),
                        _ => return Err("unexpected character"),
                    }
                }
            }
            b'0'..=b'9' => {
                let mut res = 0;

                loop {
                    res = res * 10 + (input[0] - b'0') as usize;
                    input = &input[1..];
                    if input.is_empty() || !(b'0'..=b'9').contains(&input[0]) {
                        break;
                    }
                }
                Ok((Expression::Item(res), input))
            }
            _ => Err("invalid character"),
        }
    }
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
            (Self::Item(left), Self::List(_)) => Self::List(vec![Self::Item(*left)]).cmp(other),
            (Self::List(_), Self::Item(right)) => self.cmp(&Self::List(vec![Self::Item(*right)])),
            (Self::List(left), Self::List(right)) => {
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

impl FromStr for Expression {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (expression, rest) = Expression::parse(s.as_bytes())?;
        if !rest.is_empty() {
            Err("trailing characters")?;
        }
        Ok(expression)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();

    let dividers = ["[[2]]", "[[6]]"]
        .into_iter()
        .map(str::parse)
        .collect::<Result<Vec<Expression>, _>>()?;

    let mut expressions = lines
        .flatten()
        .filter_map(|l| {
            if l.is_empty() {
                None
            } else {
                Some(l.parse().map_err(Box::from))
            }
        })
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
