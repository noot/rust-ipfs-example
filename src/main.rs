use rust_ipfs::libp2p::futures::StreamExt as _;
use rust_ipfs::{ConnectionEvents, Multiaddr};

#[tokio::main]
async fn main() {
    let bootstrap_nodes = vec![
        "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
        "/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa",
        "/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb",
        "/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
        "/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt",
        "/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
        "/ip4/104.131.131.82/udp/4001/quic-v1/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ",
    ];

    let mut builder = rust_ipfs::UninitializedIpfsDefault::new()
        .set_default_listener()
        .with_default();
    for addr in bootstrap_nodes {
        let maddr: Multiaddr = addr.parse().expect("can parse bootstrap address");
        builder = builder.add_bootstrap(maddr);
    }

    let ipfs = builder.start().await.expect("can start ipfs node");

    let addrs = ipfs
        .listening_addresses()
        .await
        .expect("can get listening addrs");
    let peer_id = ipfs.keypair().public().to_peer_id();
    println!("ipfs node started: {}  {:?}", peer_id, addrs);

    let mut conn_events = ipfs
        .connection_events()
        .await
        .expect("can get connection events");
    while let Some(event) = conn_events.next().await {
        match event {
            ConnectionEvents::IncomingConnection {
                connection_id,
                addr,
                peer_id,
            } => {
                println!(
                    "incoming connection (id {}) from {} at {}",
                    connection_id, peer_id, addr
                );
            }
            ConnectionEvents::OutgoingConnection {
                peer_id,
                connection_id,
                addr,
            } => {
                println!(
                    "outgoing connection (id {}) to {} at {}",
                    connection_id, peer_id, addr
                );
            }
            ConnectionEvents::ClosedConnection {
                peer_id,
                connection_id,
            } => {
                println!("closed connection (id {}) to {}", connection_id, peer_id);
            }
        }
    }
}
