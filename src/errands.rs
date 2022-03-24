use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter, Error, ErrorKind},
    process::exit,
};

use colored::{ColoredString, Colorize};
use rand::{seq::SliceRandom, thread_rng};
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::cli::{Location, Order, Priority};
use super::{LOCAL_PATH, PRIORITY_COLORS, USER_PATH};

const ERRANDS_PRIORITY_NOT_FOUND: &'static str =
    "Failed to find specified priority in errands list";

#[derive(Deserialize, Serialize)]
pub struct Errands {
    pub data: BTreeMap<Priority, Vec<String>>,
}

fn create_file(verbose: usize, location: &Location) -> File {
    let file = match location {
        Location::Local => {
            if verbose > 1 {
                println!(
                    "Creating errands list in {}",
                    LOCAL_PATH.to_str().unwrap().green()
                );
            }
            File::create(LOCAL_PATH.as_path())
        }
        Location::User => {
            if verbose > 1 {
                println!(
                    "Creating errands list in {}",
                    USER_PATH.to_str().unwrap().green()
                );
            }
            File::create(USER_PATH.as_path())
        }
    };

    match file {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error creating file ({:?}): {}", location, err.to_string());
            exit(1);
        }
    }
}

fn open_file(verbose: usize, truncate: bool, location: &Option<Location>) -> File {
    let file = match &location {
        Some(location) => match location {
            Location::Local => {
                if verbose > 1 {
                    println!(
                        "{}",
                        format!(
                            "Opening errands list from: {}",
                            LOCAL_PATH.to_str().unwrap().yellow()
                        )
                    );
                }
                File::options()
                    .read(true)
                    .write(true)
                    .truncate(truncate)
                    .open(LOCAL_PATH.as_path())
            }
            Location::User => {
                if verbose > 1 {
                    println!(
                        "{}",
                        format!(
                            "Opening errands list from: {}",
                            USER_PATH.to_str().unwrap().yellow()
                        )
                        .green()
                    );
                }
                File::options()
                    .read(true)
                    .write(true)
                    .truncate(truncate)
                    .open(USER_PATH.as_path())
            }
        },
        None => {
            if verbose > 1 {
                println!("{}", "List location not specified!".yellow());
            }
            if let Ok(file) = File::options()
                .read(true)
                .write(true)
                .truncate(truncate)
                .open(LOCAL_PATH.as_path())
            {
                if verbose > 1 {
                    println!(
                        "{}",
                        format!(
                            "Found errands list in: {}",
                            LOCAL_PATH.to_str().unwrap().yellow()
                        )
                        .green()
                    );
                }
                Ok(file)
            } else if let Ok(file) = File::options()
                .read(true)
                .write(true)
                .truncate(truncate)
                .open(USER_PATH.as_path())
            {
                if verbose > 1 {
                    println!(
                        "{}",
                        format!(
                            "Found errands list in: {}",
                            USER_PATH.to_str().unwrap().yellow()
                        )
                        .green()
                    );
                }
                Ok(file)
            } else {
                Err(Error::new(
                    ErrorKind::NotFound,
                    "Could not find errands list",
                ))
            }
        }
    };

    match file {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Error opening file: {}", err.to_string());
            exit(1);
        }
    }
}

impl Errands {
    pub fn new(verbose: usize, location: &Location) -> Result<Self, serde_yaml::Error> {
        let file = create_file(verbose, location);
        let mut errands = Errands {
            data: BTreeMap::new(),
        };

        errands.data.insert(Priority::Deferred, vec![]);
        errands.data.insert(Priority::Routine, vec![]);
        errands.data.insert(Priority::Medium, vec![]);
        errands.data.insert(Priority::High, vec![]);
        errands.data.insert(Priority::Urgent, vec![]);
        errands.data.insert(Priority::Emergency, vec![]);

        if verbose > 1 {
            println!("{}", "Dumping template config to file".green());
        }

        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &errands)?;
        Ok(errands)
    }

    pub fn open(verbose: usize, location: &Option<Location>) -> Result<Self, serde_yaml::Error> {
        let file = open_file(verbose, false, location);
        let reader = BufReader::new(file);
        let errands: Errands = serde_yaml::from_reader(reader)?;
        Ok(errands)
    }

    pub fn add(&mut self, verbose: usize, errand: String, priority: &Option<Priority>) {
        let priority = priority.unwrap_or(Priority::Routine);
        match self.data.get_mut(&priority) {
            Some(list) => list.push(errand.clone()),
            None => {
                self.data.insert(priority, vec![errand.clone()]);
            }
        }
        if verbose > 1 {
            println!(
                "{}",
                format!(
                    "Added '{}' to priority level: '{}'",
                    errand.white(),
                    format!("{:?}", priority).yellow()
                )
                .green()
            );
        }
    }

    pub fn clean(&mut self, _verbose: usize, priority: &Option<Priority>) {
        match priority {
            Some(priority) => {
                self.data.remove(priority);
            }
            None => self.data.clear(),
        }
    }

    pub fn list(
        &self,
        _verbose: usize,
        ignore: &Option<String>,
        order: &Option<Order>,
        priority: &Option<Priority>,
        count: &Option<usize>,
    ) -> Result<(), String> {
        let mut errands: Vec<ColoredString> = vec![];
        if let Some(priority) = priority {
            errands.extend(
                self.data
                    .get(priority)
                    .ok_or(ERRANDS_PRIORITY_NOT_FOUND)?
                    .iter()
                    .map(|errand| errand.color(*PRIORITY_COLORS.get(priority).unwrap())),
            );
        } else {
            errands.extend(self.data.iter().flat_map(|(priority, list)| {
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
            let ignore_regex = Regex::new(ignore.as_str())
                .map_err(|err| format!("Error parsing regex {}: ({})", ignore, err.to_string()))?;
            errands.retain(|errand| !ignore_regex.is_match(errand.to_string().as_str()));
        }

        if let Some(count) = count {
            errands.truncate(*count);
        }

        for errand in errands {
            println!("{}", errand);
        }

        Ok(())
    }

    pub fn remove(&mut self, _verbose: usize, priority: &Option<Priority>, errands: Vec<String>) {
        match &priority {
            Some(priority) => {
                self.data
                    .entry(*priority)
                    .or_insert(vec![])
                    .retain(|errand| !errands.contains(errand));
            }
            None => {
                self.data
                    .values_mut()
                    .for_each(|list| list.retain(|errand| !errands.contains(errand)));
            }
        }
    }

    pub fn dump(
        self,
        verbose: usize,
        truncate: bool,
        location: &Option<Location>,
    ) -> Result<(), serde_yaml::Error> {
        let file = open_file(verbose, truncate, &location);
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &self)?;
        Ok(())
    }
}
