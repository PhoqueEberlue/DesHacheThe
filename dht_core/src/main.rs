mod network;
mod rest_api;
mod sysinfo_extractor;
mod cli;
mod database;

use clap::Parser;
use std::error::Error;
use futures_util::StreamExt;
use tracing_subscriber::EnvFilter;
use tokio;
use tokio::io::{self, AsyncBufReadExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Tracing lib
    // TODO: Learn how to use it 🤓
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
    
    let opt = cli::Opt::parse();

    let (mut network_client, mut network_events, network_event_loop, peer_id) =
        network::new(opt.secret_key_seed).await?;

    // Spawn the network task for it to run in the background.
    let task_network_event_loop = tokio::task::spawn(network_event_loop.run());

    // In case a listen address was provided use it, otherwise listen on any
    // address.
    match opt.listen_address {
        Some(addr) => network_client
            .start_listening(addr)
            .await
            .expect("Listening not to fail."),
        None => network_client
            .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
            .await
            .expect("Listening not to fail."),
    };

    let database_connection = database::DatabaseConnection::new(peer_id).await.unwrap();

    // Machine info task
    let task_sysinfo = tokio::task::spawn(async move {
        loop {
            // Fetching machine informations every 10 seconds
            let _ = database_connection.add_sysinfo_record(String::from_utf8(sysinfo_extractor::get_record()).unwrap()).await;
            std::thread::sleep(std::time::Duration::from_secs(10));
        }
    });

    // tokio::task::spawn(rest_api::run());

    if opt.cli_mode {
        tokio::task::spawn(async move {
            // Read full lines from stdin
            let mut stdin = io::BufReader::new(io::stdin()).lines();

            loop {
                let line = stdin.next_line().await;
                cli::handle_input_line(line.unwrap().expect("Stdin not to close"), &mut network_client).await;
            }
        });
    }

    let task_network_event = tokio::task::spawn(async move {
        loop {
            let event = network_events.next().await;
            handle_network_event(event.expect("event")).await;
        }
    });

    let _ = tokio::join!(task_network_event, task_sysinfo, task_network_event_loop);

    Ok(())
}

async fn handle_network_event(event: network::Event) {
    match event {
        network::Event::GetResult { key, value, publisher } => {
            println!("New record: {key}, {value}, {:?}", publisher);
        }
        network::Event::PutResult { result } => {
            println!("PutResult: {:?}", result); 
        }
    }
}
