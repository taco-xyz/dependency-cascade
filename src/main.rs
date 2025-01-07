mod types;
mod commands;

use clap::Parser;
use types::DependencyGraph;
use std::{fs::File, io::BufReader, path::PathBuf};
use commands::Commands;


#[derive(Parser)]
#[command(
    version, 
    about, 
    long_about = None,
    color = clap::ColorChoice::Auto,
    styles = get_styles()
)]
struct Cli {
    /// Optional name to operate on
    #[arg(help_heading = "OPTIONS")]
    name: Option<String>,

    /// Sets a custom config file
    #[arg(
        short, 
        long, 
        value_name = "FILE",
        help_heading = "OPTIONS"
    )]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(
        short, 
        long, 
        action = clap::ArgAction::Count,
        help_heading = "GLOBAL FLAGS"
    )]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

pub fn get_styles() -> clap::builder::Styles {
    use clap::builder::styling::{Style, Color, AnsiColor};
    clap::builder::Styles::styled()
        .header(Style::new().bold().fg_color(Some(Color::Ansi(AnsiColor::Green))))
        .usage(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))))
        .error(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))))
}

fn main() {
    let cli = Cli::parse();

    // let log_level: LevelFilter = match cli.debug {
        //     0 => LevelFilter::Warn,
        //     1 => LevelFilter::Info,
        //     2 => LevelFilter::Debug,
        //     _ => LevelFilter::Trace,
        // };
        
    env_logger::builder().init();

    match cli.command {
        Some(Commands::Prepare { dir, dependency_toml_name }) => {
            
            // Prepare the graph object
            let graph = commands::prepare(dir, dependency_toml_name);
            
            // Serialize the graph object to JSON
            match graph {
                Ok(g) => match serde_json::to_string(&g) {  
                    Ok(json) => println!("{}", json),
                    Err(e) => println!("Error serializing: {}", e),
                },
                Err(e) => println!("Error: {}", e),
            }
        }
        Some(Commands::Query { graph_artifact_path, files }) => {
            // Read the graph artifact from the file
            let file = File::open(graph_artifact_path).unwrap();
            let reader = BufReader::new(file);
            let graph: DependencyGraph = serde_json::from_reader(reader).unwrap();

            // Query the graph for the given files
            let affected_nodes = commands::query(&graph, &files);

            // Serialize the affected nodes to JSON
            match serde_json::to_string(&affected_nodes) {
                Ok(json) => println!("{}", json),
                Err(e) => println!("Error serializing: {}", e),
            }
        }
        None => println!("No command provided. Use --help for more information."),
    }
}
