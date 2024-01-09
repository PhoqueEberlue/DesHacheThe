use futures::channel::{
    mpsc::{self, Receiver},
    oneshot,
};
// use tokio::sync::{mpsc, oneshot}; // Should I use tokio oneshot & mpsc instead??
//
use futures_util::{SinkExt, Stream, StreamExt};
use libp2p::{
    identity, kad, mdns, noise, request_response,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, PeerId, Swarm,
};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::result::Result;
use std::time::Duration;

// We create a custom network behaviour that combines Kademlia and mDNS.
#[derive(NetworkBehaviour)]
struct Behaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    mdns: mdns::async_io::Behaviour,
    // request_response: request_response::Behaviour<>,
}

pub(crate) struct Client {
    sender: mpsc::Sender<Command>,
}

// Commands used to control the p2p client
#[derive(Debug)]
enum Command {
    StartListening {
        addr: Multiaddr,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>, // Used to get the result
    },
    Get {
        key: Vec<u8>,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
    Put {
        key: Vec<u8>,
        value: Vec<u8>,
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
    },
}

#[derive(Debug)]
pub(crate) enum Event {
    GetResult {
        key: String,
        value: String,
        publisher: Option<String>,
    },
    PutResult {
        result: Result<(), Box<dyn Error + Send>>,
    },
}

pub(crate) struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
    pending_dial: HashMap<PeerId, oneshot::Sender<Result<(), Box<dyn Error + Send>>>>,
    pending_start_providing: HashMap<kad::QueryId, oneshot::Sender<()>>,
    pending_get_providers: HashMap<kad::QueryId, oneshot::Sender<HashSet<PeerId>>>,
}

pub(crate) async fn new(
    secret_key_seed: Option<u8>,
) -> Result<(Client, impl Stream<Item = Event>, EventLoop), Box<dyn Error>> {
    let id_keys = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(bytes).unwrap()
        }
        None => identity::Keypair::generate_ed25519(),
    };
    let peer_id = id_keys.public().to_peer_id();

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(id_keys)
        .with_async_std()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|key| {
            Ok(Behaviour {
                kademlia: kad::Behaviour::new(
                    peer_id,
                    kad::store::MemoryStore::new(key.public().to_peer_id()), // Should I use the
                                                                             // same peer id????
                ),
                mdns: mdns::async_io::Behaviour::new(
                    mdns::Config::default(),
                    key.public().to_peer_id(),
                )?,
                // request_response: request_response::cbor::Behaviour::new(
                //     [(
                //         StreamProtocol::new("/file-exchange/1"),
                //         ProtocolSupport::Full,
                //     )],
                //     request_response::Config::default(),
                // ),
            })
        })?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
        .build();

    swarm
        .behaviour_mut()
        .kademlia
        .set_mode(Some(kad::Mode::Server));

    let (command_sender, command_receiver) = mpsc::channel(0);
    let (event_sender, event_receiver) = mpsc::channel(0);

    Ok((
        Client {
            sender: command_sender,
        },
        event_receiver,
        EventLoop::new(swarm, command_receiver, event_sender),
    ))
}

impl Client {
    /// Listen for incoming connections on the given address.
    pub(crate) async fn start_listening(
        &mut self,
        addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");

        receiver.await.expect("Sender not to be dropped.")
    }

    pub(crate) async fn get(&mut self, key: Vec<u8>) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .send(Command::Get { key, sender })
            .await
            .expect("Wallahi");

        receiver.await.expect("xd")
    }

    pub(crate) async fn put(
        &mut self,
        key: Vec<u8>,
        value: Vec<u8>,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .send(Command::Put { key, value, sender })
            .await
            .expect("Command receiver not to be dropped.");

        receiver.await.expect("Sender not to be dropped.")
    }
}

impl EventLoop {
    fn new(
        swarm: Swarm<Behaviour>,
        command_receiver: mpsc::Receiver<Command>,
        event_sender: mpsc::Sender<Event>,
    ) -> Self {
        Self {
            swarm,
            command_receiver,
            event_sender,
            pending_dial: Default::default(),
            pending_start_providing: Default::default(),
            pending_get_providers: Default::default(),
        }
    }

    pub(crate) async fn run(mut self) {
        println!("Network event loop running");
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => self.handle_event(event).await,
                command = self.command_receiver.select_next_some() => self.handle_command(command).await,
            }
        }
    }

    async fn handle_event(&mut self, event: SwarmEvent<BehaviourEvent>) {
        match event {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening in {address:?}");
            }
            SwarmEvent::Behaviour(BehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                for (peer_id, multiaddr) in list {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, multiaddr);
                }
            }
            SwarmEvent::Behaviour(BehaviourEvent::Kademlia(
                kad::Event::OutboundQueryProgressed { result, .. },
            )) => match result {
                kad::QueryResult::GetRecord(Ok(kad::GetRecordOk::FoundRecord(
                    kad::PeerRecord {
                        record:
                            kad::Record {
                                key,
                                value,
                                publisher,
                                ..
                            },
                        ..
                    },
                ))) => {
                    self.event_sender
                        .send(Event::GetResult {
                            key: String::from_utf8(key.as_ref().to_vec()).unwrap(),
                            value: String::from_utf8(value).unwrap(),
                            publisher: match publisher {
                                Some(p) => Some(String::from_utf8_lossy(&p.to_bytes()).to_string()),
                                None => None,
                            },
                        })
                        .await
                        .expect("Yep");
                }
                kad::QueryResult::GetRecord(Ok(_)) => {}
                kad::QueryResult::GetRecord(Err(err)) => {
                    eprintln!("Failed to get record: {err:?}");
                }
                kad::QueryResult::PutRecord(Ok(kad::PutRecordOk { key })) => {
                    println!(
                        "Successfully put record {:?}",
                        std::str::from_utf8(key.as_ref()).unwrap()
                    );
                    self.event_sender
                        .send(Event::PutResult { result: Ok(()) })
                        .await
                        .expect("yep");
                }
                kad::QueryResult::PutRecord(Err(err)) => {
                    eprintln!("Failed to put record: {err:?}");
                }
                _ => {}
            },
            _ => {}
        }
    }

    async fn handle_command(&mut self, command: Command) {
        match command {
            Command::StartListening { addr, sender } => {
                let _ = match self.swarm.listen_on(addr) {
                    Ok(_) => sender.send(Ok(())),
                    Err(e) => sender.send(Err(Box::new(e))),
                };
            }
            Command::Get { key, sender } => {
                let key = kad::RecordKey::new(&key);

                self.swarm.behaviour_mut().kademlia.get_record(key);
                let _ = sender.send(Ok(()));
            }
            Command::Put { key, value, sender } => {
                let key = kad::RecordKey::new(&key);

                let record = kad::Record {
                    key,
                    value,
                    publisher: None,
                    expires: None,
                };

                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .put_record(record, kad::Quorum::One)
                    .expect("Failed to store record locally.");

                let _ = sender.send(Ok(()));
            }
        }
    }
}
