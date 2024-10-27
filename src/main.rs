use clap::Parser;
use std::path::PathBuf;
use opcua::client::ClientBuilder;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the client configuration file
    #[arg(short, long)]
    client_config: PathBuf,

    /// List of topics to subscribe to
    #[arg(short, long)]
    topics: Vec<String>,

    /// Namespace to use for the OPC UA server
    #[arg(short, long)]
    namespace: u16,
}

fn main() {
    let args = Args::parse();

    let client = ClientBuilder::new().with_config(args.client_config).client().expect("Failed to create client");


    
}