use clap::Parser;

use errands::{
    cli::{Args, Commands},
    errands::Errands,
};

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Add {
            location,
            priority,
            errand,
        } => {
            if args.verbose > 1 {
                println!("Adding errand {}", errand);
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
            if args.verbose > 1 {
                println!("Cleaning errands");
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
            if args.verbose > 1 {
                println!("Printing errands");
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
            if args.verbose > 1 {
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
            if args.verbose > 1 {
                println!("Removing items: {:#?}", errands);
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
