use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io::{stdin, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();

    let (n, score, _) = lines
        .flatten()
        .try_fold(
            (0, 0, HashMap::<_, usize>::new()),
            |(mut n, mut score, mut known), line| {
                for item in HashSet::<_, RandomState>::from_iter(line.chars()) {
                    *known.entry(item).or_default() += 1;
                }
                n = (n + 1) % 3;
                if n == 0 {
                    let mut filtered = known.iter().filter(|(_, count)| **count == 3);
                    let badge = *filtered.next().ok_or("No candidate was found")?.0;
                    let base = match badge {
                        'a'..='z' => 96,
                        'A'..='Z' => 38,
                        _ => Err("Invalid item")?,
                    };
                    score += u8::try_from(badge)
                        .map(|c| c as usize - base)
                        .map_err(|_| "Invalid char")?;
                    if filtered.next().is_some() {
                        Err("More than one candidate was found")?
                    }
                    known.clear()
                }
                Ok::<_, Box<dyn Error>>((n, score, known))
            },
        )?;
    if n != 0 {
        Err("Input length is not divisible by 3")?
    }

    println!("{}", score);
    Ok(())
}
