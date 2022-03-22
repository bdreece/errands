use std::{
    collections::HashMap,
    fs::File,
    io::{stdout, BufReader, BufWriter, Stdout, Write},
    path::PathBuf,
};

use clap::lazy_static::lazy_static;
use colored::{Color, Colorize};
use rand::{seq::SliceRandom, thread_rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml::{self, Result};

pub mod cli;
use cli::{Location, Order};

#[derive(Deserialize, Serialize)]
pub struct Errands(HashMap<usize, Vec<String>>);

lazy_static! {
    static ref LOCAL_PATH: PathBuf = PathBuf::from(".").join("errands.yml");
    static ref LOCAL_ERR: String = String::from("Error opening file: ./errands.yml");
    static ref USER_PATH: PathBuf = PathBuf::from("~")
        .join(".config")
        .join("errands")
        .join("errands.yml");
    static ref USER_ERR: String = String::from("Error opening file: ~/.config/errands/errands.yml");
    static ref GLOBAL_PATH: PathBuf = PathBuf::from("/")
        .join("etc")
        .join("errands")
        .join("errands.yml");
    static ref GLOBAL_ERR: String = String::from("Error opening file: /etc/errands/errands.yml");
}

fn get_file(location: &Option<Location>) -> std::io::Result<File> {
    return match &location {
        Some(location) => match location {
            Location::Local => Ok(File::open(LOCAL_PATH.as_path())?),
            Location::User => Ok(File::open(USER_PATH.as_path())?),
            Location::Global => Ok(File::open(GLOBAL_PATH.as_path())?),
        },
        None => {
            let mut some_file: Option<File> = None;
            if let Ok(file) = File::open(LOCAL_PATH.as_path()) {
                some_file = Some(file);
            } else if let Ok(file) = File::open(USER_PATH.as_path()) {
                some_file = Some(file);
            } else if let Ok(file) = File::open(GLOBAL_PATH.as_path()) {
                some_file = Some(file);
            }
            Ok(some_file.ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Errands list not found",
            ))?)
        }
    };
}

impl Errands {
    pub fn new(location: &Location) -> Self {
        let file = match location {
            Location::Local => File::create(LOCAL_PATH.as_path()).expect(LOCAL_ERR.as_str()),
            Location::User => File::create(USER_PATH.as_path()).expect(USER_ERR.as_str()),
            Location::Global => File::create(GLOBAL_PATH.as_path()).expect(GLOBAL_ERR.as_str()),
        };
        let mut errands = Errands(HashMap::new());
        errands.0.insert(0, vec![]);

        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &errands).unwrap();
        errands
    }

    pub fn open(location: &Option<Location>) -> Result<Self> {
        let file = get_file(location).unwrap();
        let reader = BufReader::new(file);
        let errands: Errands = serde_yaml::from_reader(reader)?;
        Ok(errands)
    }

    pub fn add(&mut self, errand: String, priority: &Option<usize>) {
        match priority {
            Some(priority) => match self.0.get_mut(&priority) {
                Some(list) => list.push(errand),
                None => {
                    self.0.insert(*priority, vec![errand]);
                }
            },
            None => {
                let mut min = usize::MAX;
                self.0
                    .iter_mut()
                    .fold(&mut vec![], |accum, (&priority, list)| {
                        if priority < min {
                            min = priority;
                            return list;
                        }
                        accum
                    })
                    .push(errand);
            }
        }
    }

    pub fn clean(&mut self, priority: &Option<usize>) {
        match priority {
            Some(priority) => {
                self.0.remove(priority);
            }
            None => self.0.clear(),
        }
    }

    pub fn list(
        &self,
        ignore: &Option<String>,
        order: &Option<Order>,
        priority: &Option<usize>,
        count: &Option<usize>,
    ) {
        let colors: Vec<Color> = vec![
            Color::Red,
            Color::Yellow,
            Color::Green,
            Color::Cyan,
            Color::Blue,
            Color::Magenta,
        ];
        let mut errands: Vec<String> = vec![];
        if let Some(priority) = priority {
            errands.extend(
                self.0
                    .get(priority)
                    .cloned()
                    .expect("Priority level not found"),
            );
        } else {
            errands.extend(self.0.values().flatten().cloned().collect::<Vec<String>>());
        }

        if let Some(order) = order {
            match &order {
                Order::Descending => {}
                Order::Ascending => errands.reverse(),
                Order::Random => {
                    let mut rng = thread_rng();
                    errands.shuffle(&mut rng);
                }
            }
        }

        if let Some(ignore) = ignore {
            let ignore_regex = Regex::new(ignore.as_str()).unwrap();
            errands.retain(|errand| !ignore_regex.is_match(errand.as_str()));
        }

        if let Some(count) = count {
            errands.truncate(*count);
        }

        errands
            .iter()
            .zip(colors.iter().cycle())
            .for_each(|(errand, &color)| {
                println!("{}", errand.color(color));
            });
    }

    pub fn remove(&mut self, priority: &Option<usize>, errands: Vec<String>) {
        match &priority {
            Some(priority) => {
                self.0
                    .get_mut(priority)
                    .unwrap()
                    .retain(|errand| !errands.contains(errand));
            }
            None => {
                self.0
                    .values_mut()
                    .for_each(|list| list.retain(|errand| !errands.contains(errand)));
            }
        }
    }

    pub fn dump(self, location: &Option<Location>) -> Result<()> {
        let file = match location {
            Some(Location::User) => File::create(USER_PATH.as_path()).expect(USER_ERR.as_str()),
            Some(Location::Global) => {
                File::create(GLOBAL_PATH.as_path()).expect(GLOBAL_ERR.as_str())
            }
            _ => File::create(LOCAL_PATH.as_path()).expect(LOCAL_ERR.as_str()),
        };
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &self)?;
        Ok(())
    }
}
