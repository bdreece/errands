use std::process::exit;

use clap::{lazy_static::lazy_static, Parser};
use colored::Colorize;

use errands::{
    cli::{Args, Commands},
    errands::Errands,
};

lazy_static! {
    static ref UNEXPECTED_ERR: &'static str = "Unexpected error! Aborting";
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Add {
            location,
            priority,
            errand,
        } => match Errands::open(args.verbose, &location) {
            Ok(mut errands) => {
                if args.verbose > 0 {
                    println!("{}", "Adding an errand!".cyan());
                }
                errands.add(args.verbose, errand, &priority);
                if let Err(err) = errands.dump(args.verbose, true, &location) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                exit(1);
            }
        },
        Commands::Clean { location, priority } => match Errands::open(args.verbose, &location) {
            Ok(mut errands) => {
                if args.verbose > 0 {
                    println!("{}", "Cleaning errands list!".cyan());
                }
                errands.clean(args.verbose, &priority);
                if let Err(err) = errands.dump(args.verbose, true, &location) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                exit(1);
            }
        },
        Commands::Ls {
            location,
            ignore,
            order,
            priority,
            count,
        } => match Errands::open(args.verbose, &location) {
            Ok(errands) => {
                if args.verbose > 0 {
                    println!("{}", "Listing errands!".cyan());
                }
                if let Err(err) = errands.list(args.verbose, &ignore, &order, &priority, &count) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                exit(1);
            }
        },
        Commands::Init { location } => match Errands::new(args.verbose, &location) {
            Ok(errands) => {
                if args.verbose > 0 {
                    println!("{}", "Initializing errands list!".cyan());
                }
                if let Err(err) = errands.dump(args.verbose, false, &Some(location)) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                exit(1);
            }
        },
        Commands::Rm {
            location,
            priority,
            errands,
        } => match Errands::open(args.verbose, &location) {
            Ok(mut list) => {
                if args.verbose > 0 {
                    println!("{}", "Removing errands!".cyan());
                }
                list.remove(args.verbose, &priority, errands);
                if let Err(err) = list.dump(args.verbose, true, &location) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR.red(), err.to_string());
                exit(1);
            }
        },
    }
}
