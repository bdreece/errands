use clap::{ArgEnum, Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Command line arguments
#[derive(Parser, Debug)]
#[clap(author = "Brian Reece", version = "0.1", about = "A to-do list terminal prompt", long_about = None)]
pub struct Args {
    #[clap(short, long, parse(from_occurrences))]
    pub verbose: usize,

    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(ArgEnum, Clone, Debug)]
pub enum Order {
    /// Descending in priority
    Descending,
    /// Ascending in priority
    Ascending,
    /// Random priority
    Random,
}

#[derive(Deserialize, Serialize, ArgEnum, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Emergency,
    Urgent,
    High,
    Medium,
    Routine,
    Deferred,
}

/// Errand list location
#[derive(ArgEnum, Clone, Debug)]
pub enum Location {
    /// ./errands.yml
    Local,
    /// ~/.config/errands/errands.yml
    User,
}

/// Command line subcommands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initializes errands list
    Init {
        #[clap(arg_enum)]
        location: Location,
    },
    /// Cleans errands list
    Clean {
        #[clap(short, long, arg_enum)]
        location: Option<Location>,

        #[clap(arg_enum)]
        priority: Option<Priority>,
    },
    /// Adds an item to the errands list
    Add {
        #[clap(short, long, arg_enum)]
        location: Option<Location>,

        #[clap(short, long, arg_enum)]
        priority: Option<Priority>,

        errand: String,
    },
    /// Lists errands
    List {
        #[clap(short, long, arg_enum)]
        location: Option<Location>,

        #[clap(short, long)]
        ignore: Option<String>,

        #[clap(short, long, arg_enum)]
        order: Option<Order>,

        #[clap(short, long)]
        count: Option<usize>,

        #[clap(arg_enum)]
        priority: Option<Priority>,
    },
    /// Removes errand(s)
    Rm {
        #[clap(short, long, arg_enum)]
        location: Option<Location>,

        #[clap(arg_enum)]
        priority: Option<Priority>,

        errands: Vec<String>,
    },
}
