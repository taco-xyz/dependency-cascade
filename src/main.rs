mod types;
mod commands;

use clap::{Parser, Subcommand};
use types::DependencyGraph;
use std::{path::PathBuf, time::Instant};
use log::{debug, LevelFilter};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Prepares the dependency graph from the given files
    Prepare {
        /// The directory to recursively scan for files
        #[arg(short, long, value_name = "DIR")]
        dir: PathBuf,
        /// The name of the dependency toml file commmon to all the services. Defaults to `dependencies.toml`
        #[arg(long, value_name = "NAME")]
        dependency_toml_name: Option<String>,
    },
    /// Queries the dependency graph
    Query {
        /// The JSON artifact containing the previously prepared dependency graph from the `prepare` command
        #[arg(short, long, value_name = "FILE")]
        graph_artifact: String,
        /// A list of file paths to query
        #[arg(short, long, value_name = "FILE")]
        files: Vec<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    let log_level = match cli.debug {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    env_logger::builder().init();

    match cli.command {
        Some(Commands::Prepare { dir, dependency_toml_name }) => {
            let start = Instant::now();
            let graph = commands::create_graph_from_dir(dir, dependency_toml_name);
            let end = start.elapsed();
            debug!("Time taken to prepare graph: {:?}", end);
            
            match graph {
                Ok(g) => match serde_json::to_string(&g) {  
                    Ok(json) => println!("{}", json),
                    Err(e) => println!("Error serializing: {}", e),
                },
                Err(e) => println!("Error: {}", e),
            }
        }
        Some(Commands::Query { graph_artifact, files }) => {
            let start = Instant::now();
            let graph: DependencyGraph = serde_json::from_str(&graph_artifact).unwrap();
            let end = start.elapsed();
            debug!("Time taken to deserialize graph: {:?}", end);

            let start = Instant::now();
            let affected_nodes = commands::get_affected_nodes(&graph, &files);
            let end = start.elapsed();
            debug!("Time taken to get affected nodes: {:?}", end);

            match serde_json::to_string(&affected_nodes) {
                Ok(json) => println!("{}", json),
                Err(e) => println!("Error serializing: {}", e),
            }
        }
        None => println!("No command provided. Use --help for more information."),
    }
}
