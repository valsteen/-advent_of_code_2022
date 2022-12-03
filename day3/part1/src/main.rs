use std::collections::HashSet;
use std::error::Error;
use std::io::{stdin, BufRead};

fn main() -> Result<(), Box<dyn Error>> {
    let lines = stdin().lock().lines();

    let score = lines.flatten().try_fold(0, |mut score, line| {
        if line.len() % 2 == 1 {
            Err("Not an even amount of items")?
        }
        let part1 = &line[0..line.len() / 2];
        let part2 = &line[line.len() / 2..];

        score += part1
            .chars()
            .try_fold((HashSet::new(), 0), |(mut found, score), c| {
                let base = match c {
                    'a'..='z' => 96,
                    'A'..='Z' => 38,
                    _ => Err("Invalid item")?,
                };
                if part2.contains(c) && found.insert(c) {
                    Ok::<_, Box<dyn Error>>((
                        found,
                        u8::try_from(c)
                            .map(|c| c as usize - base)
                            .map_err(|_| "Invalid char")?
                            + score,
                    ))
                } else {
                    Ok((found, score))
                }
            })?
            .1;
        Ok::<_, Box<dyn Error>>(score)
    })?;

    println!("{}", score);
    Ok(())
}
