use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::io::{stdin, Read};
use std::iter::once;
use std::num::ParseIntError;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::sync::{Arc, atomic, LockResult, Mutex, MutexGuard};
use std::sync::atomic::AtomicU16;
use std::thread;
use std::thread::available_parallelism;
use std::time::{Duration, Instant};

use crossbeam::channel::{bounded, unbounded};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, multispace0, multispace1};
use nom::combinator::{all_consuming, map_res};
use nom::error::{ContextError, ErrorKind, FromExternalError, ParseError, VerboseError};
use nom::multi::separated_list1;
use nom::sequence::tuple;
use nom::{Finish, IResult};
use rayon::prelude::*;

// 3120 = too low ?

const MAX_TIME: usize = 33;
const BLUEPRINTS: usize = 3;

trait ExpressionParseError<'a>:
    ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, ParseIntError> + Debug
{
}

impl<'a> ExpressionParseError<'a> for VerboseError<&'a str> {}

fn number<'a, E: ExpressionParseError<'a>>(i: &'a str) -> IResult<&'a str, usize, E> {
    map_res(digit1, |s: &str| s.parse())(i)
}

#[derive(Debug, Hash)]
#[repr(usize)]
enum Material {
    Ore = 0,
    Clay,
    Obsidian,
    Geode,
}

#[derive(Debug, Copy, Clone, Hash)]
struct BluePrint([Materials<4>; 4]);

impl Index<Material> for BluePrint {
    type Output = Materials<4>;

    fn index(&self, index: Material) -> &Self::Output {
        &self.0[index as usize]
    }
}

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
struct Materials<const N: usize>([usize; N]);

impl Default for Materials<4> {
    fn default() -> Self {
        Self([0, 0, 0, 0])
    }
}

impl<const N: usize> Deref for Materials<N> {
    type Target = [usize; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const N: usize> DerefMut for Materials<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const N: usize> Index<Material> for Materials<N> {
    type Output = usize;

    fn index(&self, index: Material) -> &Self::Output {
        &self[index as usize]
    }
}

impl<const N: usize> Index<usize> for Materials<N> {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const N: usize> IndexMut<usize> for Materials<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const N: usize> IndexMut<Material> for Materials<N> {
    fn index_mut(&mut self, index: Material) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

fn materials<'a, E: ExpressionParseError<'a>>(i: &'a str) -> IResult<&'a str, Materials<4>, E> {
    let (rest, list) = separated_list1(
        tag(" and "),
        tuple((
            number::<E>,
            multispace1,
            alt((tag("ore"), tag("clay"), tag("obsidian"))),
        )),
    )(i)?;

    list.into_iter()
        .try_fold(
            Materials::default(),
            |mut materials, (amount, _, material)| {
                match material {
                    "ore" if materials[Material::Ore] > 0 => Err("ore was set twice")?,
                    "ore" => {
                        materials[Material::Ore] = amount;
                    }
                    "clay" if materials[Material::Clay] > 0 => Err("clay was set twice")?,
                    "clay" => {
                        materials[Material::Clay] = amount;
                    }
                    "obsidian" if materials[Material::Obsidian] > 0 => {
                        Err("obsidian was set twice")?
                    }
                    "obsidian" => {
                        materials[Material::Obsidian] = amount;
                    }
                    _ => unreachable!(),
                };
                Ok::<_, &str>(materials)
            },
        )
        .map_err(|_| nom::Err::Failure(E::from_error_kind(rest, ErrorKind::Tag)))
        .map(|materials| (rest, materials))
}

fn blueprints<'a, E: ExpressionParseError<'a>>(i: &'a str) -> IResult<&'a str, Vec<BluePrint>, E> {
    let (rest, raw_blueprints) = separated_list1(
        multispace1,
        tuple((
            tag("Blueprint "),
            number,
            tag(":"),
            multispace1,
            tag("Each ore robot costs "),
            materials,
            tag("."),
            multispace1,
            tag("Each clay robot costs "),
            materials,
            tag("."),
            multispace1,
            tag("Each obsidian robot costs "),
            materials,
            tag("."),
            multispace1,
            tag("Each geode robot costs "),
            materials,
            tag("."),
        )),
    )(i)?;

    Ok((
        rest,
        Vec::from_iter(raw_blueprints.into_iter().map(
            |(_, _, _, _, _, ore, _, _, _, clay, _, _, _, obsidian, _, _, _, geode, _)| {
                BluePrint([ore, clay, obsidian, geode])
            },
        )),
    ))
}

#[derive(Debug, Clone, Copy)]
struct State {
    blueprint: BluePrint,
    materials: Materials<4>,
    robots: Materials<4>,
    time: usize,
}

impl Eq for State {}

impl PartialEq<Self> for State {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.blueprint.hash(state);
        self.robots.hash(state);
        self.materials.hash(state);
        self.time.hash(state);
    }
}

impl PartialOrd<Self> for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        let key = |state: &State| {
            (
                state.robots[Material::Geode] as f32 / state.time as f32,
                state.materials[Material::Geode] as f32 / state.time as f32,
                state.robots[Material::Obsidian],
                state.materials[Material::Obsidian],
                state.robots[Material::Clay],
                state.materials[Material::Clay],
                state.robots[Material::Ore],
                state.materials[Material::Ore],
                state.time,
            )
        };

        key(self).partial_cmp(&key(other)).unwrap()
    }
}

impl State {
    fn new(blueprint: BluePrint) -> Self {
        Self {
            blueprint,
            materials: Default::default(),
            robots: Materials([1, 0, 0, 0]),
            time: 0,
        }
    }

    fn next(&self) -> impl Iterator<Item = State> + '_ {
        (self.time + 1 < MAX_TIME)
            .then_some(
                self.blueprint
                    .0
                    .iter()
                    .enumerate()
                    .flat_map(|(robot_type, robot_materials)| {
                        robot_materials
                            .into_iter()
                            .enumerate()
                            .try_fold(
                                (robot_type, self.materials),
                                |(robot_type, mut remaining), (material_type, cost)| {
                                    remaining[material_type] = remaining[material_type]
                                        .checked_sub(cost)
                                        .ok_or("insufficient materials")?;
                                    Ok::<_, &'static str>((robot_type, remaining))
                                },
                            )
                            .map(|(robot_type, remaining)| (Some(robot_type), remaining))
                    })
                    .chain(once((None, self.materials)))
                    .map(move |(robot_type, mut materials)| {
                        for (material_type, remaining) in materials.into_iter().enumerate() {
                            materials[material_type] = remaining + self.robots[material_type]
                        }

                        let robots = robot_type
                            .map(|robot_type| {
                                let mut robots = self.robots;
                                robots[robot_type] += 1;
                                robots
                            })
                            .unwrap_or(self.robots);

                        State {
                            blueprint: self.blueprint,
                            materials,
                            robots,
                            time: self.time + 1,
                        }
                    }),
            )
            .into_iter()
            .flatten()
    }
}

fn main() -> Result<(), String> {
    let mut output = String::new();

    let mut blueprints = {
        stdin()
            .read_to_string(&mut output)
            .map_err(|err| err.to_string())?;
        match all_consuming(tuple((blueprints::<VerboseError<&str>>, multispace0)))(&output)
            .finish()
        {
            Ok((_, (blueprints, _))) => blueprints,
            Err(e) => {
                for err in e.errors.into_iter().map(|(err, kind)| {
                    let cr = "\n";
                    format!(r#"Could not parse:{cr}{}{cr}{:?}"#, err, kind)
                }) {
                    println!("{}", err);
                }
                return Err("Error while processing input".into());
            }
        }
    };

    blueprints.truncate(3);

    let parallelism = available_parallelism().unwrap().get();
    let (task_sender, task_receiver) = bounded::<(State, usize)>(parallelism*2);
    let (result_sender, result_receiver) = unbounded::<(State, usize, bool)>();
    let (working_sender, working_receiver) = unbounded::<i8>();

    for _ in 0..parallelism {
        let task_receiver = task_receiver.clone();
        let result_sender = result_sender.clone();
        let working_sender = working_sender.clone();
        thread::spawn(move || {
            let mut queue = vec![];
            while let Ok((state, blueprint)) = task_receiver.recv() {
                working_sender.send(1).expect("working_sender error");

                let generations = state
                    .blueprint
                    .0
                    .into_iter()
                    .enumerate()
                    .fold([0; 4], |acc, (_, materials)| {
                        [
                            acc[0].max(materials[0]),
                            acc[1].max(materials[1]),
                            acc[2].max(materials[2]),
                            acc[3].max(materials[3]),
                        ]
                    })
                    .into_iter()
                    .enumerate()
                    .map(|(material, required)| required.saturating_sub(state.materials[material]))
                    .max()
                    .unwrap_or_default()
                    .min(1);

                queue.extend(state.next().map(|state| (state, 0)));

                while let Some((state, generation)) = queue.pop() {
                    if generation < generations {
                        result_sender
                            .send((state, blueprint, false))
                            .expect("result_sender error");

                        if state.time + 2 < MAX_TIME {
                            queue.extend(state.next().map(|state| (state, generation + 1)))
                        }
                    } else {
                        result_sender
                            .send((state, blueprint, true))
                            .expect("result_sender error");
                    }
                }
                working_sender.send(-1).expect("working_sender error");
            }
        });
    }

    let mut max = [0;BLUEPRINTS];
    let mut max_robots = [0;BLUEPRINTS];
    //let mut best_at = Vec::from_iter((0..blueprints.len()).map(|_| [0; MAX_TIME]));
    let heaps : [_;BLUEPRINTS]  =  std::array::from_fn(|_| Mutex::new(BinaryHeap::<(State, usize)>::new()));

    //let mut heaps =  Vec::from_iter((0..blueprints.len()).map(|_| BinaryHeap::<(State, usize)>::new()));

    for (n, blueprint) in blueprints.into_iter().enumerate() {
        let state = State::new(blueprint);
        heaps[n].lock().unwrap().push((state, n));
    }

    let mut done = HashSet::new();
    let workers = Arc::new(AtomicU16::new(0));
    let mut instant = Instant::now();
    let mut shrink = Instant::now();
    let max = loop {
        let blocked_send = heaps
            .par_iter()
            .map_with(
                (workers.clone(), instant, shrink),
                |(workers, instant, shrink), heap| {
                    let mut heap = heap.lock().unwrap();
                    let mut blocked_send = false;
                    while let Some((state, blueprint)) = heap.pop() {
                        if Instant::now() > *instant + Duration::from_secs(5) {
                            println!("{} {:?}", blueprint, state.materials);
                            *instant = Instant::now();
                        }
                        if Instant::now() > *shrink + Duration::from_secs(60) {
                            heap.shrink_to_fit();
                            *shrink = Instant::now();
                        }

                        match task_sender.try_send((state, blueprint)) {
                            Ok(_) => {
                                //let mut workers = workers.lock().unwrap();
                                while let Ok(working) = working_receiver.try_recv() {
                                    workers.fetch_add(working as u16, atomic::Ordering::Relaxed);
                                    //*workers += working;
                                }
                            }
                            Err(_) => {
                                if blocked_send {
                                    break;
                                }
                                blocked_send = true;
                            }
                        }
                    }
                    blocked_send
                },
            )
            .any(|blocked_send| blocked_send);

        if Instant::now() > instant + Duration::from_secs(5) {
            instant = Instant::now();
        }
        if Instant::now() > shrink + Duration::from_secs(60) {
            shrink = Instant::now();
        }

        let mut received = false;

        {
            while let Ok((state, blueprint, process)) = result_receiver.try_recv() {
                received = true;

                //max[blueprint]
                //
                // match best_at[blueprint][state.time].cmp(&(state.materials[Material::Geode])) {
                //     Ordering::Less => best_at[blueprint][state.time] = state.materials[Material::Geode],
                //     Ordering::Equal => (),
                //     Ordering::Greater => {
                //         if state.materials[Material::Geode] * 2 < best_at[blueprint][state.time] {
                //             continue
                //         }
                //     },
                // }

                let candidate = state.materials[Material::Geode];
                if candidate > max[blueprint] {
                    println!("{} {}", blueprint, candidate);
                    max[blueprint] = candidate;
                }

                if state.robots[Material::Geode] > max_robots[blueprint] {
                    max_robots[blueprint] = state.robots[Material::Geode];
                } else if max_robots[blueprint].saturating_sub(MAX_TIME - state.time - 1) > state.robots[Material::Geode] {
                    continue;
                }

                if !done.contains(&(state, blueprint)) {
                    if process {
                        heaps[blueprint].lock().unwrap().push((state, blueprint));
                    }
                    done.insert((state, blueprint));
                }
            }
        }

        while let Ok(working) = working_receiver.try_recv() {
            workers.fetch_add(working as u16, atomic::Ordering::Relaxed);
        }

        if heaps.iter().all(|heap| heap.lock().unwrap().is_empty())
            && !blocked_send
            && !received
            && workers.load(atomic::Ordering::Relaxed) == 0
        {
            if let Ok(working) = working_receiver.recv_timeout(Duration::from_secs(1)) {
                workers.fetch_add(working as u16, atomic::Ordering::Relaxed);
            } else {
                break max;
            }
        }
    };

    let result = max.clone().into_iter().product::<usize>();
    println!("{:?} {}", max, result);

    Ok(())
}
