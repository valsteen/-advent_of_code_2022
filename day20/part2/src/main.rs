use std::{
    error::Error,
    io::{stdin, BufRead},
};

const FACTOR: i64 = 811589153;

fn main() -> Result<(), Box<dyn Error>> {
    let mut input = {
        let lines = stdin().lock().lines();
        lines
            .enumerate()
            .map(|(i, s)| Ok((s?.parse::<i64>()? * FACTOR, i)))
            .collect::<Result<Vec<(i64, usize)>, Box<dyn Error>>>()?
    };

    let mut destinations = input.clone();

    for n in 0..input.len() * 10 {
        let n = n % input.len();
        let (mut delta, old_pos) = input[n];

        delta %= input.len() as i64 - 1;
        if delta < 0 {
            delta += (input.len() as i64) - 1
        }

        let new_pos = (old_pos + delta as usize) % input.len();

        if old_pos == new_pos {
            continue;
        }

        let mut pull_index = old_pos;
        loop {
            let next_index = (pull_index + 1) % input.len();
            let item_to_move = destinations[next_index];

            input[item_to_move.1].1 = pull_index;

            destinations[pull_index] = item_to_move;

            if next_index == new_pos {
                input[n].1 = new_pos;

                destinations[next_index] = (input[n].0, n);
                break;
            }

            pull_index = (pull_index + 1) % input.len()
        }
    }

    let start = input.iter().find(|n| n.0 == 0).unwrap().1;

    let result = [1000usize, 2000, 3000]
        .into_iter()
        .map(|delta| destinations[(delta + start) % input.len()].0)
        .sum::<i64>();

    println!("{}", result);
    Ok(())
}
