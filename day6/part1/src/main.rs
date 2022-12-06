use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::error::Error;
use std::io::{BufRead, stdin};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();
    let positions = lines.flatten().filter_map( |line| {
        (3..line.len()).find(|pos| {
            let chars = HashSet::<_, RandomState>::from_iter(line[pos-3..=*pos].chars());
            chars.len() == 4
        })
    } ).map(|c| c+1);

    for pos in positions {
        println!("{}", pos)
    }
    Ok(())
}
