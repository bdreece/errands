use clap::lazy_static::lazy_static;
use colored::Color;
use std::{collections::BTreeMap, path::PathBuf};

pub mod cli;
pub mod errands;

use crate::cli::Priority;

lazy_static! {
    pub static ref LOCAL_PATH: PathBuf = PathBuf::from(".").join("errands.yml");
    pub static ref LOCAL_ERR: String = String::from("Error opening file: ./errands.yml");
    pub static ref USER_PATH: PathBuf = PathBuf::from("~")
        .join(".config")
        .join("errands")
        .join("errands.yml");
    pub static ref USER_ERR: String =
        String::from("Error opening file: ~/.config/errands/errands.yml");
    pub static ref GLOBAL_PATH: PathBuf = PathBuf::from("/")
        .join("etc")
        .join("errands")
        .join("errands.yml");
    pub static ref GLOBAL_ERR: String =
        String::from("Error opening file: /etc/errands/errands.yml");
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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
