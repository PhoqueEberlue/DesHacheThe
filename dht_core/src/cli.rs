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

/// Return several matrices
pub(crate) fn parse_input_data(data: String) -> Vec<(String, String)> {
    let mut res = vec![];

    let tmp: Vec<&str> = data.split(";").collect();

    for d in tmp {
        let key_val: Vec<&str> = d.split("=").collect();

        if key_val.len() < 2 { break; }

        res.push((key_val[0].to_owned(), key_val[1].to_owned()))
    }

    res
}
