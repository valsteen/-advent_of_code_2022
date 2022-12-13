use std::cmp::Ordering;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::str::FromStr;

#[derive(Debug)]
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

impl PartialEq<Self> for Expression {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd for Expression {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Self::Item(left), Self::Item(right)) => left.partial_cmp(right),
            (Self::Item(left), Self::List(_)) => {
                Self::List(vec![Self::Item(*left)]).partial_cmp(other)
            }
            (Self::List(_), Self::Item(right)) => {
                self.partial_cmp(&Self::List(vec![Self::Item(*right)]))
            }
            (Self::List(left), Self::List(right)) => {
                for (left, right) in left.iter().zip(right) {
                    match left.partial_cmp(right)? {
                        Ordering::Less => return Ordering::Less.into(),
                        Ordering::Greater => return Ordering::Greater.into(),
                        Ordering::Equal => (),
                    }
                }
                left.len().partial_cmp(&right.len())
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
    let mut lines = lines.flatten();

    let mut score = 0;
    let mut count = 0;
    while let Some(line) = lines.next() {
        count += 1;
        let expression1 = line.parse::<Expression>()?;
        let expression2 = lines.next().ok_or("unexpected end of input")?.parse()?;

        if expression1.partial_cmp(&expression2) == Some(Ordering::Less) {
            score += count;
        }

        if let Some(next) = lines.next() {
            if !next.is_empty() {
                return Err("unexpected non-empty line".into());
            }
        }
    }
    println!("{}", score);
    Ok(())
}
