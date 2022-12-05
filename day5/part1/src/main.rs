use std::collections::VecDeque;
use std::error::Error;
use std::io::{stdin, BufRead};

#[derive(Debug)]
struct Move {
    repeat: usize,
    source: usize,
    destination: usize,
}

type Stacks = Vec<VecDeque<String>>;

fn parse_input() -> Result<(Stacks, Vec<Move>), Box<dyn Error>> {
    let crates_re = regex::Regex::new(r"(?P<empty>    ?)|\[(?P<label>\w)\] ?")?;
    let lanes_re = regex::Regex::new(r"(?P<lane>[0-9]+)")?;
    let move_re = regex::Regex::new(
        r"^move (?P<repeat>[0-9]+) from (?P<source>[0-9]+) to (?P<destination>[0-9]+)$",
    )?;

    let lines = stdin().lock().lines().flatten().collect::<Vec<String>>();
    let mut stacks = Vec::<VecDeque<String>>::new();
    let mut idx = 0;

    while let Some(line) = lines.get(idx) {
        let mut matches = false;
        for (column, cap) in crates_re.captures_iter(line).enumerate() {
            if let Some(name) = cap.name("label") {
                matches = true;
                while column + 1 > stacks.len() {
                    stacks.push(Default::default())
                }
                stacks
                    .get_mut(column)
                    .ok_or("missing stack")?
                    .push_front(name.as_str().to_string());
            }
        }
        if !matches {
            break;
        }
        idx += 1;
    }

    if let Some(line) = lines.get(idx) {
        if lanes_re
            .captures_iter(line)
            .enumerate()
            .map_while(|(n, cap)| {
                cap.name("lane")
                    .filter(|label| (label.as_str().parse() == Ok(n + 1)))
            })
            .count()
            != stacks.len()
        {
            Err("Unexpected amount of lanes")?
        }
    } else {
        Err("Unexpected end of input")?
    }

    if lines.get(idx + 1).filter(|line| line.is_empty()).is_none() {
        Err("Expected empty line")?
    }

    let moves = lines
        .iter()
        .skip(idx + 2)
        .map(|line| {
            let mut captures = move_re.captures_iter(line);
            let cap = captures.next().ok_or("Unexpected format")?;
            let repeat = cap
                .name("repeat")
                .ok_or("invalid format: missing repeat")?
                .as_str()
                .parse::<usize>()?;
            let source = cap
                .name("source")
                .ok_or("invalid format: missing source")?
                .as_str()
                .parse::<usize>()?
                - 1;
            let destination = cap
                .name("destination")
                .ok_or("invalid format: missing destination")?
                .as_str()
                .parse::<usize>()?
                - 1;

            if captures.next().is_some() {
                Err("Unexpected trailing content")?
            }
            Ok(Move {
                repeat,
                source,
                destination,
            })
        })
        .collect::<Result<Vec<Move>, Box<dyn Error>>>()?;

    Ok((stacks, moves))
}

fn main() -> Result<(), Box<dyn Error>> {
    let (mut stacks, moves) = parse_input()?;
    for mov in moves {
        let stack = {
            let source = stacks.get_mut(mov.source).ok_or("unknown lane")?;
            if mov.repeat > source.len() {
                Err("Too many crates to move")?
            }
            source.split_off(source.len() - mov.repeat)
        };
        stacks
            .get_mut(mov.destination)
            .ok_or("unknown lane")?
            .extend(stack.into_iter().rev());
    }

    let result = stacks.iter().try_fold(String::new(), |result, stack| {
        Ok::<_, Box<dyn Error>>(result + stack.back().ok_or("empty stack")?)
    })?;
    println!("{:?}", result);
    Ok(())
}
