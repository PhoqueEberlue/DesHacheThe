use futures::channel::{
    mpsc::{self},
    oneshot,
};
// use tokio::sync::{mpsc, oneshot}; // Should I use tokio oneshot & mpsc instead??
//
use futures_util::{SinkExt, Stream, StreamExt};
use libp2p::{
    identity, kad, mdns, noise,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux, Multiaddr, Swarm, PeerId,
};
use std::error::Error;
use std::result::Result;
use std::time::Duration;

// Custom network behaviour for libp2p using kademlia DHT and mDNS behaviours
#[derive(NetworkBehaviour)]
struct Behaviour {
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    mdns: mdns::async_io::Behaviour,
}

/// Setup network instance using libp2p
/// Parameter:
/// - secret_key_seed: seed for generating the keys of the local node
///
/// Returns:
/// - Client: Struct providing functions to interact with the network
/// - Event receiver: communicate network events through a mpsc channel
/// - EventLoop: Struct that contains the run function in order to handle network events
pub(crate) async fn new(
    secret_key_seed: Option<u8>,
) -> Result<(Client, impl Stream<Item = Event>, EventLoop, String), Box<dyn Error>> {
    let id_keys = match secret_key_seed {
        Some(seed) => {
            let mut bytes = [0u8; 32];
            bytes[0] = seed;
            identity::Keypair::ed25519_from_bytes(bytes).unwrap()
        }
        None => identity::Keypair::generate_ed25519(),
    };
    let peer_id = id_keys.public().to_peer_id();
    println!("{}", peer_id);

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
        peer_id.to_string()
    ))
}

/// Client providing methods to interact with the network
pub(crate) struct Client {
    sender: mpsc::Sender<Command>,
}

impl Client {
    /// Send a command to listen for incoming connections on the given address.
    pub(crate) async fn start_listening(
        &mut self,
        addr: Multiaddr,
    ) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();

        // Send command to EventLoop
        self.sender
            // pass the sender we created as argument in order to receive a result
            .send(Command::StartListening { addr, sender })
            .await
            .expect("Command receiver not to be dropped.");

        // Waiting result from the EventLoop
        receiver.await.expect("Sender not to be dropped.")
    }

    /// Send get command to retrieve a kademlia record
    pub(crate) async fn get(&mut self, key: Vec<u8>) -> Result<(), Box<dyn Error + Send>> {
        let (sender, receiver) = oneshot::channel();

        self.sender
            .send(Command::Get { key, sender })
            .await
            .expect("Sender not to be dropped");

        receiver.await.expect("xd")
    }

    /// Send put command to put a kademlia record
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

// Commands passed between Client and EventLoop in order to communicate
#[derive(Debug)]
enum Command {
    StartListening {
        addr: Multiaddr,
        // Oneshot channel that lets the EventLoop sent a result to the Client
        sender: oneshot::Sender<Result<(), Box<dyn Error + Send>>>,
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

/// EventLoop handles network behaviour events and commands by the Client 
pub(crate) struct EventLoop {
    swarm: Swarm<Behaviour>,
    command_receiver: mpsc::Receiver<Command>,
    event_sender: mpsc::Sender<Event>,
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
        }
    }

    /// Handles events and commands
    pub(crate) async fn run(mut self) {
        println!("Network event loop running");
        loop {
            tokio::select! {
                command = self.command_receiver.select_next_some() => self.handle_command(command).await,
                event = self.swarm.select_next_some() => self.handle_event(event).await,
            }
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
                            publisher,
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
}

/// Events that the EventLoop sends via the event sender 
#[derive(Debug)]
pub(crate) enum Event {
    GetResult {
        key: String,
        value: String,
        publisher: Option<PeerId>,
    },
    PutResult {
        result: Result<(), Box<dyn Error + Send>>,
    },
}
