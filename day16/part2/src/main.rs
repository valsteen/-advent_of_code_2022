use std::cmp::Ordering;
use std::collections::hash_map::{Entry, RandomState};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::{stdin, BufRead};
use std::ops::{Deref, Neg};
use std::str::{from_utf8, FromStr};

use nom::character::complete::alphanumeric1;
use nom::multi::separated_list1;
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

trait ExpressionParseError<'a, T: FromStr>:
    ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, <T as FromStr>::Err>
{
}
impl<'a, T: FromStr> ExpressionParseError<'a, T> for VerboseError<&'a str> {}

fn number<'a, E: ExpressionParseError<'a, T>, T: FromStr>(i: &'a str) -> IResult<&'a str, T, E> {
    map_res(digit1, |s: &str| s.parse::<T>())(i)
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct Label([u8; 2]);

impl Deref for Label {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        from_utf8(&self.0).unwrap()
    }
}

impl Debug for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.deref(), f)
    }
}

impl Label {
    fn new(value: [u8; 2]) -> Self {
        Self(value)
    }
}

#[derive(Debug)]
struct Valve {
    label: Label,
    rate: usize,
    destinations: Vec<(usize, Label)>,
}

trait Distance {
    fn distance(&self, destination: Label) -> usize;
}

impl Distance for Vec<(usize, Label)> {
    fn distance(&self, destination: Label) -> usize {
        self.iter().find(|(_,label)| *label == destination).unwrap().0
    }
}

fn line<'a, E: ExpressionParseError<'a, usize>>(i: &'a str) -> IResult<&'a str, Valve, E> {
    let (_, (_, valve, _, rate, _, valves)) = all_consuming(tuple((
        tag("Valve "),
        alphanumeric1,
        tag(" has flow rate="),
        number,
        alt((
            tag("; tunnels lead to valves "),
            tag("; tunnel leads to valve "),
        )),
        separated_list1(tag(", "), alphanumeric1),
    )))(i)?;

    let valve = valve.as_bytes();
    Ok((
        i,
        Valve {
            label: Label::new([valve[0], valve[1]]),
            rate,
            destinations: valves
                .into_iter()
                .map(|valve| {
                    let valve = valve.as_bytes();
                    (1, Label::new([valve[0], valve[1]]))
                })
                .collect(),
        },
    ))
}

#[derive(Clone)]
struct State {
    my_position: Label,
    elephant: Label,
    score: usize,
    opened: HashSet<Label>,
    time: usize,
    my_move: usize,
    elephant_move: usize,
}

const TIMEOUT: usize = 25;

impl State {
    fn forward(&mut self, valves: &HashMap<Label, Valve>, time: usize) {
        for valve in &self.opened {
            self.score += valves.get(valve).unwrap().rate * time
        }
        self.time += time
    }

    fn next(mut self, valves: &HashMap<Label, Valve>) -> Vec<State> {
        if self.time >= TIMEOUT {
            return vec![];
        }

        let mut me_action = if self.my_move > 0 {
            self.my_move -= 1;
            true
        } else {
            false
        };

        let mut elephant_action = if self.elephant_move > 0 {
            self.elephant_move -= 1;
            true
        } else {
            false
        };

        if !me_action && !self.opened.contains(&self.my_position) {
            self.opened.insert(self.my_position);
            me_action = true;
        }

        if !elephant_action && !self.opened.contains(&self.elephant) {
            self.opened.insert(self.elephant);
            elephant_action = true;
        }

        if me_action && elephant_action {
            self.forward(valves, 1);
            return vec![self];
        }

        if self.opened.len() < valves.len() {
            let mut result = Vec::new();

            let my_current = valves.get(&self.my_position).unwrap();
            let elephant_current = valves.get(&self.elephant).unwrap();

            let mut my_destinations = vec![];
            let mut elephant_destinations = vec![];

            if !me_action {
                for (distance, destination) in &my_current.destinations {
                    if !self.opened.contains(destination) && self.time + distance <= TIMEOUT {
                        my_destinations.push(*destination);
                    }
                }
            }

            if !elephant_action {
                for (distance, destination) in &elephant_current.destinations {
                    if !self.opened.contains(destination) && self.time + distance <= TIMEOUT {
                        elephant_destinations.push(*destination);
                    }
                }
            }

            if my_destinations.is_empty() {
                my_destinations.push(self.my_position)
            }

            if elephant_destinations.is_empty() {
                elephant_destinations.push(self.elephant)
            }

            for my_destination in my_destinations {
                for elephant_destination in &elephant_destinations {
                    let mut new_state = self.clone();
                    if !me_action {
                        new_state.my_move = my_current.destinations.distance(my_destination) - 1;
                        new_state.my_position = my_destination;
                    }
                    if !elephant_action {
                        new_state.elephant_move = elephant_current.destinations.distance(*elephant_destination) - 1;
                        new_state.elephant = *elephant_destination;
                    }

                    new_state.forward(valves, 1);
                    result.push(new_state);
                }
            }

            if !result.is_empty() {
                return result;
            }
        }

        self.forward(valves, TIMEOUT - self.time);
        vec![self]
    }
}

impl PartialEq<Self> for State {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for State {}

impl PartialOrd<Self> for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score
            .cmp(&self.score)
            .reverse()
            .then_with(|| (self.time).cmp(&(other.time)))
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let lines = lines.flatten().map(|s| {
        line::<VerboseError<_>>(&s)
            .finish()
            .map(|(_, line)| line)
            .map_err(|err| err.to_string())
    });

    let mut valves = lines
        .map(|line| {
            let line = line?;
            Ok((line.label, line))
        })
        .collect::<Result<HashMap<Label, Valve>, String>>()?;

    let start = Label::new([b'A', b'A']);
    let mut destinations: HashMap<_, _, RandomState> =
        HashMap::from_iter(valves.keys().map(|k| ((*k, *k), 1)));
    let mut journeys = Vec::from([vec![start]]);
    while let Some(journey) = journeys.pop() {
        let mut to_visit = HashSet::new();
        let current = journey.last().unwrap();
        let neighbours = &valves.get(current).unwrap().destinations;
        for (distance, valve) in journey.iter().copied().rev().enumerate() {
            for (_, destination) in neighbours {
                match destinations.entry((valve, *destination)) {
                    Entry::Occupied(entry) => {
                        let entry = entry.into_mut();
                        if *entry > distance + 1 {
                            *entry = distance + 1;
                            to_visit.insert(*destination);
                        }
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(distance + 1);
                        to_visit.insert(*destination);
                    }
                };
            }
        }

        for visit in to_visit {
            let mut new_journey = journey.clone();
            new_journey.push(visit);
            journeys.push(new_journey);
        }
    }

    for valve in valves.values_mut() {
        valve.destinations = destinations
            .iter()
            .filter_map(|((source, destination), distance)| {
                if source == &valve.label {
                    Some((*distance, *destination))
                } else {
                    None
                }
            })
            .collect();
    }

    let mut max = 0;

    let mut states = BinaryHeap::from([State {
        elephant: start,
        my_position: start,
        score: 0,
        opened: valves
            .values()
            .filter_map(|valve| {
                if valve.rate == 0 {
                    Some(valve.label)
                } else {
                    None
                }
            })
            .collect(),
        time: 0,
        elephant_move: 0,
        my_move: 0
    }]);

    while let Some(next) = states.pop() {
        for state in next.next(&valves) {
            if state.score > max {
                max = state.score;
                println!("{}", max)
            }
            states.push(state)
        }
    }

    println!("{:?}", max);
    Ok(())
}
