use rust_ipfs::block::BlockCodec;
use rust_ipfs::libp2p::futures::StreamExt as _;
use rust_ipfs::{ConnectionEvents, ConnectionLimits};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Data {
    content: String,
}

#[tokio::main]
async fn main() {
    let conn_limits = ConnectionLimits::default().with_max_established(Some(100));
    let builder = rust_ipfs::UninitializedIpfsDefault::new()
        .set_default_listener()
        .with_default()
        .set_connection_limits(conn_limits)
        .set_listening_addrs(vec![
            "/ip4/0.0.0.0/tcp/5001".parse().expect("valid multiaddr"),
            "/ip4/0.0.0.0/udp/5001/quic-v1"
                .parse()
                .expect("valid multiaddr"),
        ])
        .listen_as_external_addr()
        .with_upnp();

    let ipfs = builder.start().await.expect("can start ipfs node");
    ipfs.default_bootstrap()
        .await
        .expect("can add default bootstrap nodes");
    ipfs.bootstrap().await.expect("can bootstrap IPFS node");
    ipfs.dht_mode(rust_ipfs::DhtMode::Auto)
        .await
        .expect("can set dht mode to auto");

    let addrs = ipfs
        .listening_addresses()
        .await
        .expect("can get listening addrs");
    let peer_id = ipfs.keypair().public().to_peer_id();
    println!("ipfs node started: {}  {:?}", peer_id, addrs);

    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio::task::spawn({
        let ipfs = ipfs.clone();
        async move {
            loop {
                let conn_count = ipfs
                    .connected()
                    .await
                    .expect("can get connected peers")
                    .len();

                if conn_count > 10 {
                    let _ = tx.send(());
                    return;
                }

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    });

    let _ = rx.await;
    {
        let put = ipfs.put_dag(b"nootwashere\n").codec(BlockCodec::Raw);
        let cid = put.await.expect("can put data into ipfs repo");
        println!("put data into ipfs repo: {:?}", cid);

        ipfs.provide(cid)
            .await
            .expect("can provide data in IPFS repo");
        ipfs.bootstrap().await.expect("can bootstrap IPFS node");

        let get = ipfs
            .get_dag(cid)
            .await
            .expect("can get data from IPFS repo");
        if let ipld_core::ipld::Ipld::Bytes(data) = get {
            println!(
                "got data from ipfs: {}",
                std::str::from_utf8(&data).expect("valid utf8")
            );
        } else {
            panic!("expected bytes data, got: {:?}", get);
        }

        let maddrs = ipfs
            .external_addresses()
            .await
            .expect("can get external addresses");
        println!("external addresses: {:?}", maddrs);
    }

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
