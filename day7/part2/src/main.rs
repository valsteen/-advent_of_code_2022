use recap::Recap;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, BufRead};
use std::str::FromStr;

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^\$ ls$"#)]
struct Ls {}

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"cd (?P<name>.+)"#)]
struct Cd {
    name: String,
}

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^dir (?P<name>.+)$"#)]
struct Dir {
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize, Recap)]
#[recap(regex = r#"^(?P<size>[0-9]+) (?P<name>.+)$"#)]
struct File {
    size: usize,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug)]
enum Line {
    Ls(Ls),
    Cd(Cd),
    Dir(Dir),
    File(File),
}

impl FromStr for Line {
    type Err = recap::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse()
            .map(Self::Ls)
            .or_else(|_| s.parse().map(Self::Cd))
            .or_else(|_| s.parse().map(Self::Dir))
            .or_else(|_| s.parse().map(Self::File))
    }
}

#[derive(Default, Debug)]
struct State {
    curdir: Vec<String>,
    dirsizes: HashMap<String, usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let state = lines.flatten().map(|line| line.parse()).try_fold(
        State::default(),
        |mut state, line| {
            match line? {
                Line::Dir(_) | Line::Ls(_) => (),
                Line::Cd(cd) => match cd.name.as_str() {
                    ".." => {
                        state.curdir.pop();
                    }
                    _ => state.curdir.push(cd.name),
                },
                Line::File(file) => {
                    for parts in 0..=state.curdir.len() {
                        *state
                            .dirsizes
                            .entry(state.curdir[..parts].join("/"))
                            .or_default() += file.size
                    }
                }
            };
            Ok::<_, Box<dyn Error>>(state)
        },
    )?;

    let result = state.dirsizes.get("/").ok_or("no root directory")?;
    let free = 70000000 - *result;
    let need = 30000000 - free;

    let result = state
        .dirsizes
        .iter()
        .filter_map(|(_, size)| (*size >= need).then_some(size))
        .min()
        .ok_or("no candidate")?;

    println!("{:?}", result);
    Ok(())
}
