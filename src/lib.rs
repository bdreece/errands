use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use clap::lazy_static::lazy_static;
use colored::{Color, ColoredString, Colorize};
use rand::{seq::SliceRandom, thread_rng};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_yaml::{self, Result};

pub mod cli;
use cli::{Location, Order, Priority};

#[derive(Deserialize, Serialize)]
pub struct Errands(BTreeMap<Priority, Vec<String>>);

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
    static ref PRIORITY_COLORS: BTreeMap<Priority, Color> = {
        let mut m = BTreeMap::new();
        m.insert(Priority::Emergency, Color::BrightWhite);
        m.insert(Priority::Urgent, Color::Red);
        m.insert(Priority::High, Color::Yellow);
        m.insert(Priority::Medium, Color::Green);
        m.insert(Priority::Routine, Color::Cyan);
        m.insert(Priority::Deferred, Color::Magenta);
        m
    };
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
        let mut errands = Errands(BTreeMap::new());
        errands.0.insert(Priority::Deferred, vec![]);
        errands.0.insert(Priority::Routine, vec![]);
        errands.0.insert(Priority::Medium, vec![]);
        errands.0.insert(Priority::High, vec![]);
        errands.0.insert(Priority::Urgent, vec![]);
        errands.0.insert(Priority::Emergency, vec![]);

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

    pub fn add(&mut self, errand: String, priority: &Option<Priority>) {
        let priority = priority.unwrap_or(Priority::Routine);
        match self.0.get_mut(&priority) {
            Some(list) => list.push(errand),
            None => {
                self.0.insert(priority, vec![errand]);
            }
        }
    }

    pub fn clean(&mut self, priority: &Option<Priority>) {
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
        priority: &Option<Priority>,
        count: &Option<usize>,
    ) {
        let mut errands: Vec<ColoredString> = vec![];
        if let Some(priority) = priority {
            errands.extend(
                self.0
                    .get(priority)
                    .unwrap()
                    .iter()
                    .map(|errand| errand.color(*PRIORITY_COLORS.get(priority).unwrap())),
            );
        } else {
            errands.extend(self.0.iter().flat_map(|(priority, list)| {
                list.iter()
                    .map(|errand| errand.color(*PRIORITY_COLORS.get(priority).unwrap()))
            }));
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
            errands.retain(|errand| !ignore_regex.is_match(errand.to_string().as_str()));
        }

        if let Some(count) = count {
            errands.truncate(*count);
        }

        for errand in errands {
            println!("{}", errand);
        }
    }

    pub fn remove(&mut self, priority: &Option<Priority>, errands: Vec<String>) {
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
