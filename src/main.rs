use clap::Parser;

use errands::{
    cli::{Args, Commands},
    Errands,
};

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Add {
            location,
            priority,
            errand,
        } => {
            if args.verbose > 0 {
                println!("Adding errand {}", errand);
            }
            if args.verbose > 1 {
                println!("Using errands list in location: {:?}", location);
                if let Some(priority) = priority {
                    println!("Adding with priority: {:?}", priority);
                }
            }
            let mut errands = Errands::open(&location).unwrap();
            errands.add(errand, &priority);
            errands.dump(&location).unwrap();
        }
        Commands::Clean { location, priority } => {
            if args.verbose > 0 {
                println!("Cleaning errands");
            }
            if args.verbose > 1 {
                println!("Using errands list in location: {:?}", location);
                if let Some(priority) = priority {
                    println!("Cleaning with priority: {:?}", priority);
                }
            }
            let mut errands = Errands::open(&location).unwrap();
            errands.clean(&priority);
            errands.dump(&location).unwrap();
        }
        Commands::List {
            location,
            ignore,
            order,
            priority,
            count,
        } => {
            if args.verbose > 0 {
                println!("Printing errands");
            }
            if args.verbose > 1 {
                println!("Using errands list in location: {:?}", location);
                if let Some(pattern) = &ignore {
                    println!("Ignoring pattern: {}", pattern);
                }
                if let Some(order) = &order {
                    println!("Printing in order: {:?}", order);
                }
                if let Some(priority) = &priority {
                    println!("Printing errands with priority: {:?}", priority);
                }
                if let Some(count) = &count {
                    println!("Printing with count: {}", count);
                }
            }
            let errands = Errands::open(&location).unwrap();
            errands.list(&ignore, &order, &priority, &count);
        }
        Commands::Init { location } => {
            if args.verbose > 0 {
                println!("Initializing errands in location: {:?}", location);
            }
            let errands = Errands::new(&location);
            errands.dump(&Some(location)).unwrap();
        }
        Commands::Rm {
            location,
            priority,
            errands,
        } => {
            if args.verbose > 0 {
                println!("Removing items: {:#?}", errands);
            }
            if args.verbose > 1 {
                println!("Using errands list in location: {:?}", location);
                if let Some(priority) = priority {
                    println!("Removing with priority: {:?}", priority);
                }
            }
            let mut list = Errands::open(&location).unwrap();
            list.remove(&priority, errands);
            list.dump(&location).unwrap();
        }
    }
}
