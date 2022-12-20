use std::{
    cmp::{Ordering},
    collections::BinaryHeap,
    fmt::Debug,
    hash::{Hash, Hasher},
    io::{stdin, Read},
    iter::once_with,
    num::{NonZeroUsize, ParseIntError},
    ops::{Deref, DerefMut, Index, IndexMut},
    sync::atomic::AtomicU16,
    sync::{atomic, Arc, Mutex},
    thread,
    thread::available_parallelism,
    time::{Duration, Instant},
};

use crossbeam::channel::{bounded, unbounded};
use lru::LruCache;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0, multispace1},
    combinator::{all_consuming, map_res},
    error::{ContextError, ErrorKind, FromExternalError, ParseError, VerboseError},
    multi::separated_list1,
    sequence::tuple,
    Finish, IResult,
};
use rayon::prelude::*;

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
                state.robots[Material::Geode],
                state.time,
                state.materials[Material::Geode],
                state.robots[Material::Obsidian],
                state.materials[Material::Obsidian],
                state.robots[Material::Clay],
                state.materials[Material::Clay],
                state.robots[Material::Ore],
                state.materials[Material::Ore],
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
                        if robot_materials
                            .iter()
                            .enumerate()
                            .any(|(material_type, cost)| {
                                *cost > 0 && self.robots[material_type] == 0
                            })
                        {
                            return None;
                        }

                        let mut add_time = 1;

                        let mut materials = self.materials;

                        let mut materials = loop {
                            if let Ok(remaining) = robot_materials.into_iter().enumerate().try_fold(
                                materials,
                                |mut remaining, (material_type, cost)| {
                                    remaining[material_type] =
                                        remaining[material_type].checked_sub(cost).ok_or(())?;
                                    Ok::<_, ()>(remaining)
                                },
                            ) {
                                break remaining;
                            } else {
                                for (material_type, remaining) in materials.iter_mut().enumerate() {
                                    *remaining += self.robots[material_type]
                                }
                                add_time += 1;
                            };
                        };

                        let mut robots = self.robots;
                        robots[robot_type] += 1;

                        for (material_type, remaining) in materials.iter_mut().enumerate() {
                            *remaining += self.robots[material_type]
                        }

                        Some(State {
                            blueprint: self.blueprint,
                            materials,
                            robots,
                            time: self.time + add_time,
                        })
                    })
                    .chain(once_with(|| {
                        let add_time = MAX_TIME - 1 - self.time;

                        let mut materials = self.materials;

                        for (material_type, remaining) in materials.iter_mut().enumerate() {
                            *remaining += self.robots[material_type] * add_time
                        }

                        State {
                            blueprint: self.blueprint,
                            materials,
                            robots: self.robots,
                            time: self.time + add_time,
                        }
                    })),
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
    let (task_sender, task_receiver) = bounded::<(State, usize)>(parallelism * 2);
    let (result_sender, result_receiver) = unbounded::<(State, usize)>();
    let (working_sender, working_receiver) = unbounded::<i8>();

    for _ in 0..parallelism {
        let task_receiver = task_receiver.clone();
        let result_sender = result_sender.clone();
        let working_sender = working_sender.clone();
        thread::spawn(move || {
            while let Ok((state, blueprint)) = task_receiver.recv() {
                working_sender.send(1).expect("working_sender error");
                for state in state.next() {
                    result_sender
                        .send((state, blueprint))
                        .expect("result_sender error");
                }
                working_sender.send(-1).expect("working_sender error");
            }
        });
    }

    let mut max = [0; BLUEPRINTS];
    let heaps: [_; BLUEPRINTS] =
        std::array::from_fn(|_| Mutex::new(BinaryHeap::<(State, usize)>::new()));

    for (n, blueprint) in blueprints.into_iter().enumerate() {
        let state = State::new(blueprint);
        heaps[n].lock().unwrap().push((state, n));
    }

    let mut done = LruCache::<(State, usize), bool>::new(NonZeroUsize::new(40000000).unwrap());

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
                                while let Ok(working) = working_receiver.try_recv() {
                                    workers.fetch_add(working as u16, atomic::Ordering::Relaxed);
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
            while let Ok((state, blueprint)) = result_receiver.try_recv() {
                received = true;
                if MAX_TIME < state.time + 1 {
                    continue;
                }

                let candidate = state.materials[Material::Geode];
                if candidate > max[blueprint] {
                    println!("{} {} {}", state.time, blueprint, candidate);
                    max[blueprint] = candidate;
                }

                if !done.contains(&(state, blueprint)) {
                    heaps[blueprint].lock().unwrap().push((state, blueprint));
                    done.push((state, blueprint), true);
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

    let result = max
        .into_iter()
        .filter(|x| *x > 0)
        .product::<usize>();
    println!("{:?} {}", max, result);

    Ok(())
}
