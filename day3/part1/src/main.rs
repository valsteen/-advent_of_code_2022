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
            .try_fold((None, 0), |(mut found, mut score), c| {
                let base = match c {
                    'a'..='z' => 96,
                    'A'..='Z' => 38,
                    _ => Err("Invalid item")?,
                };
                if part2.contains(c) {
                    score += match found {
                        Some(i) if i == c => 0,
                        None => u8::try_from(c)
                            .map(|c| c as usize - base)
                            .map_err(|_| "Invalid char")?,
                        _ => Err("Another item was found twice")?,
                    };
                    found = Some(c)
                }
                Ok::<_, Box<dyn Error>>((found, score))
            })?
            .1;
        Ok::<_, Box<dyn Error>>(score)
    })?;

    println!("{}", score);
    Ok(())
}
