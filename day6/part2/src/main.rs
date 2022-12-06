use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::error::Error;
use std::io::{stdin, BufRead};

const LEN: usize = 14;

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let positions = lines
        .flatten()
        .filter_map(|line| {
            (LEN - 1..line.len()).find(|pos| {
                HashSet::<_, RandomState>::from_iter(line[pos + 1 - LEN..=*pos].chars()).len()
                    == LEN
            })
        })
        .map(|c| c + 1);

    for pos in positions {
        println!("{}", pos)
    }
    Ok(())
}
