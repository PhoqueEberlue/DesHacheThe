mod network;
mod rest_api;
mod sysinfo_extractor;

use std::{error::Error, path::PathBuf};
use futures_util::StreamExt;
use network::Client;
use tracing_subscriber::EnvFilter;
use clap::Parser;
use libp2p::Multiaddr;
use tokio;
use tokio::io::{self, AsyncBufReadExt};


#[derive(Parser, Debug)]
#[clap(name = "libp2p file sharing example")]
struct Opt {
    /// Fixed value to generate deterministic peer ID.
    #[clap(long)]
    secret_key_seed: Option<u8>,

    #[clap(long)]
    peer: Option<Multiaddr>,

    #[clap(long)]
    listen_address: Option<Multiaddr>,

    // #[clap(subcommand)]
    // argument: CliArgument,
}

#[derive(Debug, Parser)]
enum CliArgument {
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    
       let opt = Opt::parse();

    let (mut network_client, mut network_events, network_event_loop) =
        network::new(opt.secret_key_seed).await?;

    // Spawn the network task for it to run in the background.
    tokio::task::spawn(network_event_loop.run());

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

    // Read full lines from stdin
    let mut stdin = io::BufReader::new(io::stdin()).lines();

    // Kick it off.
    loop {
        tokio::select! {
            line = stdin.next_line() => handle_input_line(line.unwrap().expect("Stdin not to close"), &mut network_client).await,
            event = network_events.next() => handle_network_event(event.expect("event")).await,
        }
    }
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


async fn handle_input_line(line: String, network_client: &mut Client) {
    let mut args = line.split(' ');
    match args.next() {
        Some("GET") => {
            let key = {
                match args.next() {
                    Some(key) => key,
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            };

            let _ = network_client.get(key.as_bytes().to_owned()).await;
        },
        Some("PUT") => {
            let key = {
                match args.next() {
                    Some(k) => k,
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            }; 

            let value = {
                match args.next() {
                    Some(v) => v,
                    None => {
                        eprintln!("Expected key");
                        return;
                    }
                }
            }; 

            let _ = network_client.put(key.as_bytes().to_owned(), value.as_bytes().to_owned()).await;
        }
        _ => {}
    }
}
