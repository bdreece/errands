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
                    "{} {}",
                    "Creating errands list in".green(),
                    LOCAL_PATH.to_str().unwrap().white()
                );
            }
            File::create(LOCAL_PATH.as_path())
        }
        Location::User => {
            if verbose > 1 {
                println!(
                    "{} {}",
                    "Creating errands list in".green(),
                    USER_PATH.to_str().unwrap().white()
                );
            }
            File::create(USER_PATH.as_path())
        }
    };

    match file {
        Ok(file) => file,
        Err(err) => {
            eprintln!(
                "{} ({:?}): {}",
                "Error creating file".red(),
                location,
                err.to_string().white()
            );
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
                        "{} {}",
                        "Opening errands list from:".green(),
                        LOCAL_PATH.to_str().unwrap().white()
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
                        "{} {}",
                        "Opening errands list from:".green(),
                        USER_PATH.to_str().unwrap().white()
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
                        "{} {}",
                        "Found errands list in:".green(),
                        LOCAL_PATH.to_str().unwrap().white()
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
                        "{} {}",
                        "Found errands list in: {}".green(),
                        USER_PATH.to_str().unwrap().white()
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
            eprintln!(
                "{} {}",
                "Error opening file:".red(),
                err.to_string().white()
            );
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
                    format!("{:?}", priority).white()
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
        verbose: usize,
        ignore: &Option<String>,
        order: &Option<Order>,
        priority: &Option<Priority>,
        count: &Option<usize>,
    ) -> Result<(), String> {
        let mut errands: Vec<ColoredString> = vec![];
        if let Some(priority) = priority {
            if verbose > 1 {
                println!(
                    "{} {:?}",
                    "Listing errands with priority:".green(),
                    priority
                );
            }
            errands.extend(
                self.data
                    .get(priority)
                    .ok_or(ERRANDS_PRIORITY_NOT_FOUND)?
                    .iter()
                    .map(|errand| errand.color(*PRIORITY_COLORS.get(priority).unwrap())),
            );
        } else {
            if verbose > 1 {
                println!("{}", "Listing errands of all priorities".green());
            }
            errands.extend(self.data.iter().flat_map(|(priority, list)| {
                list.iter()
                    .map(|errand| errand.color(*PRIORITY_COLORS.get(priority).unwrap()))
            }));
        }

        if let Some(order) = order {
            if verbose > 1 {
                println!("{} {:?}", "Listing errands in order:".green(), order);
            }
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
            if verbose > 1 {
                println!("{} {}", "Ignoring regex string:".green(), ignore.white());
            }
            let ignore_regex = Regex::new(ignore.as_str())
                .map_err(|err| format!("Error parsing regex {}: ({})", ignore, err.to_string()))?;
            errands.retain(|errand| !ignore_regex.is_match(errand.to_string().as_str()));
        }

        if let Some(count) = count {
            if verbose > 1 {
                println!(
                    "{} {}",
                    "Truncating list to size:".green(),
                    count.to_string().white()
                );
            }
            errands.truncate(*count);
        }

        println!("{}", "Errands:".green());
        for errand in errands {
            println!("- {}", errand);
        }

        Ok(())
    }

    pub fn remove(&mut self, verbose: usize, priority: &Option<Priority>, errands: Vec<String>) {
        match &priority {
            Some(priority) => {
                if verbose > 1 {
                    println!(
                        "{} {:?}",
                        "Removing errands with priority:".green(),
                        priority
                    );
                }
                self.data
                    .entry(*priority)
                    .or_insert(vec![])
                    .retain(|errand| !errands.contains(errand));
            }
            None => {
                if verbose > 1 {
                    println!("{} {:?}", "Removing errands by name:".green(), errands);
                }
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
        if verbose > 1 {
            println!("{}", "Dumping template config to file".green());
        }
        let file = open_file(verbose, truncate, &location);
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &self)?;
        if verbose > 1 {
            println!("{}", "Successfully wrote to errands list!".cyan());
        }
        Ok(())
    }
}
