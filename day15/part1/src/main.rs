use std::collections::{BTreeSet, HashSet};
use std::io::{BufRead, stdin};
use std::ops::Neg;
use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map_res},
    error::{ContextError, FromExternalError, ParseError, VerboseError},
    Finish,
    IResult, sequence::tuple,
};
use nom::error::VerboseErrorKind;

trait Number: FromStr + Neg<Output = Self> {}

impl<T> Number for T where T: FromStr + Neg<Output = Self> {}

trait ExpressionParseError<'a, T: FromStr>:
    ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, <T as FromStr>::Err>
{
}
impl<'a, T: FromStr> ExpressionParseError<'a, T> for VerboseError<&'a str> {}

fn number<'a, E: ExpressionParseError<'a, T>, T: Number>(i: &'a str) -> IResult<&'a str, T, E> {
    alt((
        map_res(digit1, |s: &str| s.parse::<T>()),
        map_res(tuple((tag("-"), digit1::<&str, _>)), |(_, s)| {
            s.parse::<T>().map(|v| -v)
        }),
    ))(i)
}

trait Intervals {
    fn add(&mut self, start: i32, end: i32);
    fn count(&self) -> usize;
}

impl Intervals for BTreeSet<(i32, i32)> {
    fn add(&mut self, start: i32, end: i32) {
        let mut min = start;
        let mut max = end;

        self.retain(|(start1, end1)| {
            if *end1 >= start && *start1 <= end {
                min = min.min(*start1);
                max = max.max(*end1);
                false
            } else {
                true
            }
        });
        self.insert((min, max));
    }

    fn count(&self) -> usize {
        self.iter()
            .copied()
            .map(|(start, end)| (end - start + 1) as usize)
            .sum()
    }
}

fn line<'a, T: Number, E: ExpressionParseError<'a, T>>(
    i: &'a str,
) -> IResult<&'a str, (T, T, T, T), E> {
    let (_, (_, x1, _, y1, _, x2, _, y2)) = all_consuming(tuple((
        tag("Sensor at x="),
        number,
        tag(", y="),
        number,
        tag(": closest beacon is at x="),
        number,
        tag(", y="),
        number,
    )))(i)?;

    Ok((i, (x1, y1, x2, y2)))
}

const ROW: i32 = 2000000;

fn main() {
    let lines = stdin().lock().lines();
    let lines = lines.flatten().enumerate().map(|(n, s)| {
        line::<i32, VerboseError<_>>(&s)
            .finish()
            .map(|(_, line)| line)
            .map_err(|err| {
                (
                    n,
                    err.errors
                        .into_iter()
                        .map(|(remain, e)| (remain.to_string(), e))
                        .collect::<Vec<(String, VerboseErrorKind)>>(),
                )
            })
    });

    let mut beacons = HashSet::new();

    let mut intervals = BTreeSet::new();
    for line in lines {
        let (x1, y1, x2, y2) = match line {
            Ok(line) => line,
            Err((n, errs)) => {
                println!("Error at line {}:", n);
                for (input, r) in errs {
                    println!("\t'{}': {:?}", input, r)
                }
                return;
            }
        };
        if y1 == ROW {
            beacons.insert(x1);
        }
        if y2 == ROW {
            beacons.insert(x2);
        }
        let distance = (x1 - x2).abs() + (y1 - y2).abs();
        let remains = distance - (y1 - ROW).abs();
        if remains >= 0 {
            intervals.add(x1 - remains, x1 + remains);
        }
    }

    println!("{}", intervals.count() - beacons.len());
}
