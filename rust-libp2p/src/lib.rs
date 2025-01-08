use libp2p::core::{multiaddr::Protocol, multiaddr::Multiaddr, muxing::StreamMuxerBox, transport::Boxed};
use libp2p::identity::{secp256k1, Keypair};
use libp2p::{gossipsub, identify, identity, core, noise, ping, yamux, PeerId, Transport, SwarmBuilder};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use std::time::Duration;
use futures::future::Either;
use std::{error::Error, net::Ipv4Addr,collections::hash_map::DefaultHasher,hash::{Hash, Hasher},};
use futures::StreamExt;
use slog::{crit, debug, info, o, trace, warn};
use tokio::{io, io::AsyncBufReadExt, select};

type BoxedTransport = Boxed<(PeerId, StreamMuxerBox)>;


#[no_mangle]
pub fn startNetwork(selfPort: i32, connectPort: i32) {
    build_network(selfPort, connectPort);
}

extern "C" {
    fn zig_add(a: i32, b: i32) -> i32;
}


#[derive(NetworkBehaviour)]
struct Behaviour {
    identify: identify::Behaviour,
    ping: ping::Behaviour,
    gossipsub: gossipsub::Behaviour,
}

impl Behaviour {
    fn new(key: identity::Keypair) -> Self {
        let local_public_key = key.public();
        // To content-address message, we can take the hash of message and use it as an ID.
        let message_id_fn = |message: &gossipsub::Message| {
            let mut s = DefaultHasher::new();
            message.data.hash(&mut s);
            gossipsub::MessageId::from(s.finish().to_string())
        };

        // Set a custom gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
        // .heartbeat_interval(Duration::from_secs(10)) // This is set to aid debugging by not cluttering the log space
        .validation_mode(gossipsub::ValidationMode::Strict) // This sets the kind of message validation. The default is Strict (enforce message
        // signing)
        .message_id_fn(message_id_fn) // content-address messages. No two messages of the same content will be propagated.
        .build().unwrap();
        // .map_err(|msg| io::Error::new(io::ErrorKind::Other, msg))?; // Temporary hack because `build` does not return a proper `std::error::Error`.

        // build a gossipsub network behaviour
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        ).unwrap();

        Self {
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/0.1.0".into(),
                local_public_key.clone(),
            )),
            ping: ping::Behaviour::default(),
            gossipsub,
        }
    }
}

#[tokio::main]
async fn build_network(selfPort: i32, connectPort: i32) {
    let local_private_key = secp256k1::Keypair::generate();
    let local_keypair:Keypair = local_private_key.into();
    let transport = build_transport(local_keypair.clone(), false).unwrap();
    println!("build the transport");

    let connection_limits = {
        let limits = libp2p::connection_limits::ConnectionLimits::default()
            .with_max_pending_incoming(Some(5))
            .with_max_pending_outgoing(Some(16))
            .with_max_established_incoming(Some(10))
            .with_max_established_outgoing(Some(10))
            .with_max_established(Some(10))
            .with_max_established_per_peer(Some(1));

        libp2p::connection_limits::Behaviour::new(limits)
    };

    let builder = SwarmBuilder::with_existing_identity(local_keypair)
        .with_tokio()
        .with_other_transport(|_key| transport)
        .expect("infalible");
    let mut swarm = builder
    .with_behaviour(|key| Behaviour::new(key.clone())).unwrap()
    .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
    .build();

    swarm.listen_on(
        Multiaddr::empty()
            .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(selfPort as u16)),
    );

    println!("going for loop match");

    if(connectPort > 0){
        let connectString = format!("/ip4/127.0.0.1/tcp/{}", connectPort);
        let addr: Multiaddr = connectString.parse().unwrap();

        // helper closure for dialing peers
        let mut dial = |mut multiaddr: Multiaddr| {
            // strip the p2p protocol if it exists
            strip_peer_id(&mut multiaddr);
            match swarm.dial(multiaddr.clone()) {
                Ok(()) => println!("Dialing libp2p peer address: {multiaddr}"),
                Err(err) => {
                    println!("Could not connect to peer address: {multiaddr} error: {err}");
                }
            };
        };

        dial(addr.clone());
        println!("spinning on {selfPort} and connecting on {connectPort}");
    }else{
        println!("spinning on {selfPort} and standing by...");
    }

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                let result = unsafe {zig_add(23,42)};
                println!("Listening on {address:?} result {result}");
            },
            SwarmEvent::Behaviour(event) => {
                let result = unsafe {zig_add(23,42)};
                println!("{event:?} result {result}");
            },
            e => println!("{e:?}"),
        }
    }
}

fn build_transport(
    local_private_key: Keypair,
    quic_support: bool,
) -> std::io::Result<BoxedTransport> {
    // mplex config
    let mut mplex_config = libp2p_mplex::MplexConfig::new();
    mplex_config.set_max_buffer_size(256);
    mplex_config.set_max_buffer_behaviour(libp2p_mplex::MaxBufferBehaviour::Block);

    // yamux config
    let yamux_config = yamux::Config::default();
    // Creates the TCP transport layer
    let tcp = libp2p::tcp::tokio::Transport::new(libp2p::tcp::Config::default().nodelay(true))
        .upgrade(core::upgrade::Version::V1)
        .authenticate(generate_noise_config(&local_private_key))
        .multiplex(core::upgrade::SelectUpgrade::new(
            yamux_config,
            mplex_config,
        ))
        .timeout(Duration::from_secs(10));
    let transport = if quic_support {
        // Enables Quic
        // The default quic configuration suits us for now.
        let quic_config = libp2p::quic::Config::new(&local_private_key);
        let quic = libp2p::quic::tokio::Transport::new(quic_config);
        let transport = tcp
            .or_transport(quic)
            .map(|either_output, _| match either_output {
                Either::Left((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
                Either::Right((peer_id, muxer)) => (peer_id, StreamMuxerBox::new(muxer)),
            });
        transport.boxed()
    } else {
        tcp.boxed()
    };

    // Enables DNS over the transport.
    let transport = libp2p::dns::tokio::Transport::system(transport)?.boxed();

    Ok(transport)
}

/// Generate authenticated XX Noise config from identity keys
fn generate_noise_config(identity_keypair: &Keypair) -> noise::Config {
    noise::Config::new(identity_keypair).expect("signing can fail only once during starting a node")
}

/// For a multiaddr that ends with a peer id, this strips this suffix. Rust-libp2p
/// only supports dialing to an address without providing the peer id.
fn strip_peer_id(addr: &mut Multiaddr) {
    let last = addr.pop();
    match last {
        Some(Protocol::P2p(_)) => {}
        Some(other) => addr.push(other),
        _ => {}
    }
}