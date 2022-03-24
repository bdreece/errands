use clap::lazy_static::lazy_static;
use colored::Color;
use dirs::config_dir;
use std::{collections::BTreeMap, path::PathBuf};

pub mod cli;
pub mod errands;

use crate::cli::Priority;

lazy_static! {
    pub static ref LOCAL_PATH: PathBuf = PathBuf::from(".").join("errands.yml");
    pub static ref USER_PATH: PathBuf = config_dir().unwrap().join("errands").join("errands.yml");
    pub static ref GLOBAL_PATH: PathBuf = PathBuf::from("/")
        .join("etc")
        .join("errands")
        .join("errands.yml");
    pub static ref PRIORITY_COLORS: BTreeMap<Priority, Color> = {
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

#[cfg(test)]
mod tests {
    use super::{cli::*, errands::Errands};
    use clap::lazy_static::lazy_static;
    use std::{fs::remove_file, path::PathBuf};

    lazy_static! {
        static ref PRIORITIES: Vec<Priority> = {
            vec![
                Priority::Emergency,
                Priority::Urgent,
                Priority::High,
                Priority::Medium,
                Priority::Routine,
                Priority::Deferred,
            ]
        };
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn construction() {
        let errands = Errands::new(0, &Location::Local).unwrap();
        assert_eq!(errands.data.keys().count(), 6);

        errands
            .data
            .keys()
            .zip(PRIORITIES.iter())
            .for_each(|(found, expected)| assert_eq!(found, expected));
    }

    #[test]
    fn dump() {
        let errands = Errands::new(0, &Location::Local).unwrap();
        errands.dump(0, true, &Some(Location::Local)).unwrap();
        let errands = Errands::open(0, &Some(Location::Local)).unwrap();
        assert_eq!(errands.data.keys().count(), 6);

        errands
            .data
            .keys()
            .zip(PRIORITIES.iter())
            .for_each(|(found, expected)| assert_eq!(found, expected));

        remove_file(PathBuf::from(".").join("errands.yml")).unwrap();
    }

    #[test]
    fn add() {
        let mut errands = Errands::new(0, &Location::Local).unwrap();
        errands.add(0, String::from("Something"), &Some(Priority::Emergency));
        assert!(errands
            .data
            .get(&Priority::Emergency)
            .unwrap()
            .contains(&String::from("Something")));
    }

    #[test]
    fn remove() {
        let mut errands = Errands::new(0, &Location::Local).unwrap();
        errands.add(0, String::from("Something"), &None);
        errands.remove(0, &None, vec![String::from("Something")]);
        assert!(errands.data.values().all(|list| list.is_empty()));
    }
}
