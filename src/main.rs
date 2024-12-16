use clap::Parser;
use log::{error, info};
use opcua::client::ClientBuilder;
use opcua::client::DataChangeCallback;
use opcua::client::MonitoredItem;
use opcua::client::Session;
use opcua::types::{DataValue, StatusCode};
use opcua::types::{MonitoredItemCreateRequest, NodeId, TimestampsToReturn};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Connecting to {:?}", args.client_config);
    let mut _client = ClientBuilder::from_config(&args.client_config)
        .expect(&format!(
            "Failed to create client builder from file: {:?}",
            args.client_config
        ))
        .client()
        .expect("Failed to create client");

    println!("Connecting to endpoint");
    let (session, event_loop) = _client
        .connect_to_endpoint_id(None)
        .await
        .expect("Failed to connect to endpoint");

    println!("Spawning event loop");
    let handle = event_loop.spawn();

    println!("Waiting for connection");
    session.wait_for_connection().await;

    println!("Subscribing to variables");
    if let Err(result) = subscribe_to_variables(session.clone(), args.namespace, args.topics).await
    {
        println!(
            "ERROR: Got an error while subscribing to variables - {}",
            result
        );
        let _ = session.disconnect().await;
    }

    println!("Waiting for event loop");
    handle.await.unwrap();

    Ok(())
}

pub async fn subscribe_to_variables(
    session: Arc<Session>,
    ns: u16,
    items: Vec<String>,
) -> Result<(), StatusCode> {
    // Creates a subscription with a data change callback
    let subscription_id = session
        .create_subscription(
            Duration::from_secs(1),
            10,
            30,
            0,
            0,
            true,
            DataChangeCallback::new(|dv, item| {
                print_value(&dv, item);
            }),
        )
        .await?;
    println!("Created a subscription with id = {}", subscription_id);

    // Create some monitored items
    let items_to_create: Vec<MonitoredItemCreateRequest> = items
        .iter()
        .map(|v| NodeId::new(ns, v.clone()).into())
        .collect();
    let _ = session
        .create_monitored_items(subscription_id, TimestampsToReturn::Both, items_to_create)
        .await?;

    Ok(())
}

pub fn print_value(data_value: &DataValue, item: &MonitoredItem) {
    let node_id = &item.item_to_monitor().node_id;
    if let Some(ref value) = data_value.value {
        println!(
            "[{}] {} = {:?}",
            chrono::Local::now().format("%H:%M:%S.%3f"),
            node_id,
            value
        );
    } else {
        println!(
            "Item \"{}\", Value not found, error: {}",
            node_id,
            data_value.status.as_ref().unwrap()
        );
    }
}
