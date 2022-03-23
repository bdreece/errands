use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter, Error, ErrorKind, Result},
};

use colored::{ColoredString, Colorize};
use rand::{seq::SliceRandom, thread_rng};
use regex::Regex;
use serde::{Deserialize, Serialize};

use super::cli::{Location, Order, Priority};
use super::{GLOBAL_PATH, LOCAL_PATH, PRIORITY_COLORS, USER_PATH};

#[derive(Deserialize, Serialize)]
pub struct Errands(BTreeMap<Priority, Vec<String>>);

fn create_file(verbose: usize, location: &Location) -> Result<File> {
    let file = match location {
        Location::Local => {
            if verbose > 1 {
                println!(
                    "{}",
                    "Creating errands list in current working directory".green()
                );
            }
            File::create(LOCAL_PATH.as_path())?
        }
        Location::User => {
            if verbose > 1 {
                println!(
                    "{}",
                    "Creating errands list in user config directory".green()
                );
            }
            File::create(USER_PATH.as_path())?
        }
        Location::Global => {
            if verbose > 1 {
                println!("{}", "Creating errands list in global directory".green());
            }
            File::create(GLOBAL_PATH.as_path())?
        }
    };
    Ok(file)
}

fn open_file(verbose: usize, truncate: bool, location: &Option<Location>) -> Result<File> {
    return match &location {
        Some(location) => match location {
            Location::Local => {
                if verbose > 1 {
                    println!(
                        "{}",
                        "Opening errands list from current working directory".green()
                    );
                }
                Ok(File::options()
                    .read(true)
                    .write(true)
                    .truncate(truncate)
                    .open(LOCAL_PATH.as_path())?)
            }
            Location::User => {
                if verbose > 1 {
                    println!(
                        "{}",
                        "Opening errands list from user config directory".green()
                    );
                }
                Ok(File::options()
                    .read(true)
                    .write(true)
                    .truncate(truncate)
                    .open(USER_PATH.as_path())?)
            }
            Location::Global => {
                if verbose > 1 {
                    println!("{}", "Opening errands list from global directory".green());
                }
                Ok(File::options()
                    .read(true)
                    .write(true)
                    .truncate(truncate)
                    .open(GLOBAL_PATH.as_path())?)
            }
        },
        None => {
            if verbose > 1 {
                println!("{}", "List location not specified!".yellow());
            }
            let mut some_file: Option<File> = None;
            if let Ok(file) = File::options()
                .read(true)
                .write(true)
                .truncate(truncate)
                .open(LOCAL_PATH.as_path())
            {
                if verbose > 1 {
                    println!(
                        "{}",
                        "Found errands list in current working directory".green()
                    );
                }
                some_file = Some(file);
            } else if let Ok(file) = File::options()
                .read(true)
                .write(true)
                .truncate(truncate)
                .open(USER_PATH.as_path())
            {
                if verbose > 1 {
                    println!("{}", "Found errands list in user config directory".green());
                }
                some_file = Some(file);
            } else if let Ok(file) = File::options()
                .read(true)
                .write(true)
                .truncate(truncate)
                .open(GLOBAL_PATH.as_path())
            {
                if verbose > 1 {
                    println!("{}", "Found errands list in global directory".green());
                }
                some_file = Some(file);
            }
            Ok(some_file.ok_or(Error::new(ErrorKind::NotFound, "Errands list not found"))?)
        }
    };
}

impl Errands {
    pub fn new(verbose: usize, location: &Location) -> Self {
        let file = create_file(verbose, location).unwrap();
        let mut errands = Errands(BTreeMap::new());
        errands.0.insert(Priority::Deferred, vec![]);
        errands.0.insert(Priority::Routine, vec![]);
        errands.0.insert(Priority::Medium, vec![]);
        errands.0.insert(Priority::High, vec![]);
        errands.0.insert(Priority::Urgent, vec![]);
        errands.0.insert(Priority::Emergency, vec![]);

        if verbose > 1 {
            println!("{}", "Dumping template config to file".green());
        }

        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &errands).unwrap();
        errands
    }

    pub fn open(verbose: usize, location: &Option<Location>) -> serde_yaml::Result<Self> {
        let file = open_file(verbose, false, location).unwrap();
        let reader = BufReader::new(file);
        let errands: Errands = serde_yaml::from_reader(reader)?;
        Ok(errands)
    }

    pub fn add(&mut self, verbose: usize, errand: String, priority: &Option<Priority>) {
        let priority = priority.unwrap_or(Priority::Routine);
        match self.0.get_mut(&priority) {
            Some(list) => list.push(errand.clone()),
            None => {
                self.0.insert(priority, vec![errand.clone()]);
            }
        }
        if verbose > 1 {
            println!(
                "{}",
                format!(
                    "Added '{}' to priority level: '{:?}'",
                    errand.white(),
                    priority
                )
                .green()
            );
        }
    }

    pub fn clean(&mut self, _verbose: usize, priority: &Option<Priority>) {
        match priority {
            Some(priority) => {
                self.0.remove(priority);
            }
            None => self.0.clear(),
        }
    }

    pub fn list(
        &self,
        _verbose: usize,
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

    pub fn remove(&mut self, _verbose: usize, priority: &Option<Priority>, errands: Vec<String>) {
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

    pub fn dump(
        self,
        verbose: usize,
        truncate: bool,
        location: &Option<Location>,
    ) -> serde_yaml::Result<()> {
        let file = open_file(verbose, truncate, &location).unwrap();
        let writer = BufWriter::new(file);
        serde_yaml::to_writer(writer, &self)?;
        Ok(())
    }
}
