use std::error::Error;
use std::io::{stdin, BufRead};
use std::iter::once_with;

fn rotations<T>(cubes: &'_ T) -> impl Iterator<Item = T> + '_
where
    T: 'static + AsMut<[[u8; 3]]> + Clone,
{
    once_with(|| cubes.clone())
        .chain(once_with(|| {
            let mut cubes = cubes.clone();
            cubes
                .as_mut()
                .iter_mut()
                .for_each(|c| *c = [c[2], c[0], c[1]]);
            cubes
        }))
        .chain(once_with(|| {
            let mut cubes = cubes.clone();
            cubes
                .as_mut()
                .iter_mut()
                .for_each(|c| *c = [c[1], c[2], c[0]]);
            cubes
        }))
}

fn touches(left: [u8; 3], right: [u8; 3]) -> bool {
    rotations(&[left, right]).any(|cubes| {
        cubes[0][0] == cubes[1][0]
            && cubes[0][1] == cubes[1][1]
            && cubes[0][2].abs_diff(cubes[1][2]) == 1
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let cubes: Vec<[u8; 3]> = {
        let lines = stdin().lock().lines();
        lines
            .flatten()
            .map(|line| {
                line.split(',')
                    .map(str::parse)
                    .enumerate()
                    .try_fold([0; 3], |mut acc, (i, v)| {
                        if i > 2 {
                            return Err("trailing content".into());
                        }
                        acc[i] = v?;
                        Ok(acc)
                    })
            })
            .collect::<Result<_, Box<dyn Error>>>()?
    };

    let groups = cubes
        .into_iter()
        .fold(vec![], |mut acc: Vec<(Vec<[u8; 3]>, usize)>, cube| {
            let mut merge_with = acc.iter_mut().filter_map(|(group, touching)| {
                let count = group
                    .iter()
                    .filter(|group_cube| touches(cube, **group_cube))
                    .count();
                (count > 0).then_some(((group, touching), count))
            });

            if let Some(((first, touching_first), touching)) = merge_with.next() {
                first.push(cube);
                *touching_first += touching;

                for ((merger, merger_touching), touching) in merge_with {
                    first.append(merger);
                    *touching_first += *merger_touching + touching
                }

                acc.retain(|(cubes, _)| !cubes.is_empty());
            } else {
                acc.push((vec![cube], 0));
            }
            acc
        });
    let result = groups
        .into_iter()
        .map(|(group, touching)| group.len() * 6 - touching * 2)
        .sum::<usize>();
    println!("{}", result);

    Ok(())
}
