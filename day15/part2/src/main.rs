use std::io::{stdin, BufRead};
use std::ops::Neg;
use std::str::FromStr;

use nom::error::VerboseErrorKind;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{all_consuming, map_res},
    error::{ContextError, FromExternalError, ParseError, VerboseError},
    sequence::tuple,
    Finish, IResult,
};

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

#[derive(Debug)]
struct Sensor {
    location: (i64, i64),
    distance: i64,
}

impl Sensor {
    fn new(location: (i64, i64), beacon: (i64, i64)) -> Self {
        Self {
            location,
            distance: (location.0 - beacon.0).abs() + (location.1 - beacon.1).abs(),
        }
    }
    fn zero_crossings(&self) -> ((i64, i64), (i64, i64)) {
        let (ascending, descending) =
            zero_crossing(self.location.0 - self.distance, self.location.1);
        let (ascending1, descending1) =
            zero_crossing(self.location.0 + self.distance, self.location.1);
        ((ascending, ascending1), (descending, descending1))
    }

    fn intersection_neighbours(&self, other: &Self) -> Vec<(i64, i64)> {
        let (self_ascending, self_descending) = self.zero_crossings();
        let (other_ascending, other_descending) = other.zero_crossings();
        let mut intersection_neighbours = [
            intersection(self_ascending.0, other_descending.0),
            intersection(self_ascending.1, other_descending.0),
            intersection(self_ascending.0, other_descending.1),
            intersection(self_ascending.1, other_descending.1),
            intersection(other_ascending.0, self_descending.0),
            intersection(other_ascending.1, self_descending.0),
            intersection(other_ascending.0, self_descending.1),
            intersection(other_ascending.1, self_descending.1),
        ]
        .into_iter()
        .flat_map(|(x, y)| get_neighbours(x, y))
        .collect::<Vec<_>>();
        intersection_neighbours.sort();
        intersection_neighbours.dedup();
        intersection_neighbours
    }
}

// intersection of two perpendicular lines at 45 deg, first ascending, second descending, having
// their x=0 at that given y
fn intersection(y1: i64, y2: i64) -> (f64, f64) {
    // crossings are correct, now check intersections
    ((y1 - y2) as f64 / 2., y1 as f64 - ((y1 - y2) as f64 / 2.))
}

// give the two y position at x=zero, for the two lines crossing at x,y , first ascending, second descending
fn zero_crossing(x: i64, y: i64) -> (i64, i64) {
    (x + y, y - x)
}

fn get_neighbours(x: f64, y: f64) -> Vec<(i64, i64)> {
    if x.fract() != 0. {
        [
            (x.floor() as i64, y.floor() as i64),
            (x.floor() as i64, y.ceil() as i64),
            (x.ceil() as i64, y.floor() as i64),
            (x.ceil() as i64, y.ceil() as i64),
        ]
        .into()
    } else {
        [
            (x as i64 - 1, y as i64 - 1),
            (x as i64 - 1, y as i64 + 1),
            (x as i64 + 1, y as i64 - 1),
            (x as i64 + 1, y as i64 + 1),
            (x as i64 - 1, y as i64),
            (x as i64 + 1, y as i64),
            (x as i64, y as i64 - 1),
            (x as i64, y as i64 + 1),
        ]
        .into()
    }
}

fn first_match(position: (i64, i64), sensors: &Vec<Sensor>) -> Option<&Sensor> {
    for sensor in sensors {
        let distance =
            (position.0 - sensor.location.0).abs() + (position.1 - sensor.location.1).abs();
        if distance <= sensor.distance {
            return Some(sensor);
        }
    }

    None
}

const DIM: i64 = 4000000;

fn main() {
    let lines = stdin().lock().lines();
    let lines = lines.flatten().enumerate().map(|(n, s)| {
        line::<i64, VerboseError<_>>(&s)
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

    let mut sensors = Vec::new();

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

        sensors.push(Sensor::new((x1, y1), (x2, y2)));
    }

    for sensor1 in &sensors {
        for sensor2 in sensors.iter().skip(1) {
            for (x, y) in sensor1.intersection_neighbours(sensor2) {
                if !(0..=DIM).contains(&x) || !(0..=DIM).contains(&y) {
                    continue;
                }
                if first_match((x, y), &sensors).is_none() {
                    let result = x * 4000000 + y;
                    println!("{}", result);
                    return;
                }
            }
        }
        let (ascending, descending) = sensor1.zero_crossings();
        for (x, y) in [
            (0, ascending.0),
            (0, ascending.1),
            (ascending.0, 0),
            (ascending.1, 0),
            (0, descending.0),
            (0, descending.1),
            (-descending.0, 0),
            (-descending.1, 0),
        ] {
            if !(0..=DIM).contains(&x) || !(0..=DIM).contains(&y) {
                continue;
            }

            if first_match((x, y), &sensors).is_none() {
                let result = x * 4000000 + y;
                println!("{}", result);
                return;
            }
        }
    }
}
