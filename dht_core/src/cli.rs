use clap::Parser;
use libp2p::Multiaddr;
use crate::network::Client;

#[derive(Parser, Debug)]
#[clap(name = "libp2p file sharing example")]
pub(crate) struct Opt {
    /// Fixed value to generate deterministic peer ID.
    #[arg(long)]
    pub secret_key_seed: Option<u8>,

    #[arg(long)]
    pub data: Option<String>,

    #[arg(long)]
    pub peer: Option<Multiaddr>,

    #[arg(long)]
    pub listen_address: Option<Multiaddr>,

    #[arg(long)]
    pub cli_mode: bool

    // #[clap(subcommand)]
    // argument: CliArgument,
}

#[derive(Debug, Parser)]
enum CliArgument {
}

pub(crate) async fn handle_input_line(line: String, network_client: &mut Client) {
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

pub(crate) fn parse_input_data() {
}
