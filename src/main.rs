use std::process::exit;

use clap::Parser;

use errands::{
    cli::{Args, Commands},
    errands::Errands,
};

const UNEXPECTED_ERR: &'static str = "Unexpected error! Aborting";

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Add {
            location,
            priority,
            errand,
        } => match Errands::open(args.verbose, &location) {
            Ok(mut errands) => {
                errands.add(args.verbose, errand, &priority);
                if let Err(err) = errands.dump(args.verbose, true, &location) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                exit(1);
            }
        },
        Commands::Clean { location, priority } => match Errands::open(args.verbose, &location) {
            Ok(mut errands) => {
                errands.clean(args.verbose, &priority);
                if let Err(err) = errands.dump(args.verbose, true, &location) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                exit(1);
            }
        },
        Commands::List {
            location,
            ignore,
            order,
            priority,
            count,
        } => match Errands::open(args.verbose, &location) {
            Ok(errands) => {
                if let Err(err) = errands.list(args.verbose, &ignore, &order, &priority, &count) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                exit(1);
            }
        },
        Commands::Init { location } => match Errands::new(args.verbose, &location) {
            Ok(errands) => {
                if let Err(err) = errands.dump(args.verbose, false, &Some(location)) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                exit(1);
            }
        },
        Commands::Rm {
            location,
            priority,
            errands,
        } => match Errands::open(args.verbose, &location) {
            Ok(mut list) => {
                list.remove(args.verbose, &priority, errands);
                if let Err(err) = list.dump(args.verbose, true, &location) {
                    eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                    exit(1);
                }
            }
            Err(err) => {
                eprintln!("{}: ({})", UNEXPECTED_ERR, err.to_string());
                exit(1);
            }
        },
    }
}
