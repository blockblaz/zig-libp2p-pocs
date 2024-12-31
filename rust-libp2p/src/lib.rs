use libp2p::core::{multiaddr::Protocol, multiaddr::Multiaddr, muxing::StreamMuxerBox, transport::Boxed};
use libp2p::identity::{secp256k1, Keypair};
use libp2p::{identify, identity, core, noise, yamux, PeerId, Transport, SwarmBuilder};
use libp2p::swarm::{NetworkBehaviour, SwarmEvent};
use std::time::Duration;
use futures::future::Either;
use std::{error::Error, net::Ipv4Addr};
use futures::StreamExt;

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
}

impl Behaviour {
    fn new(local_public_key: identity::PublicKey) -> Self {
        Self {
            identify: identify::Behaviour::new(identify::Config::new(
                "/ipfs/0.1.0".into(),
                local_public_key.clone(),
            )),
        }
    }
}

#[tokio::main]
async fn build_network(selfPort: i32, connectPort: i32) {
    let local_private_key = secp256k1::Keypair::generate();
    let local_keypair:Keypair = local_private_key.into();
    let transport = build_transport(local_keypair.clone(), false).unwrap();
    println!("build the transport");

    let builder = SwarmBuilder::with_existing_identity(local_keypair)
        .with_tokio()
        .with_other_transport(|_key| transport)
        .expect("infalible");
    let mut swarm = builder.with_behaviour(|key| Behaviour::new(key.public())).unwrap()
    .build();

    swarm.listen_on(
        Multiaddr::empty()
            .with(Protocol::Ip4(Ipv4Addr::UNSPECIFIED))
            .with(Protocol::Tcp(selfPort as u16)),
    );

    println!("going for loop match");

    if(connectPort > 0){
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
            SwarmEvent::Behaviour(event) => println!("{event:?}"),
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
