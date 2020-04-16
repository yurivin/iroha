use crate::isi::Contract;
use crate::{tx::Transaction, Id};
use futures::{future::FutureExt, lock::Mutex, pin_mut, select};
use iroha_derive::*;
use iroha_network::{prelude::*, Network};
use parity_scale_codec::{Decode, Encode};
use std::{
    collections::{HashMap, HashSet},
    convert::TryInto,
    sync::Arc,
    time::{Duration, SystemTime},
};

type PublicKey = [u8; 32];

pub mod isi {
    use super::*;
    use iroha_derive::{IntoContract, Io};

    /// The purpose of add peer command is to write into ledger the fact of peer addition into the
    /// peer network. After a transaction with AddPeer has been committed, consensus and
    /// synchronization components will start using it.
    #[derive(Clone, Debug, PartialEq, Io, IntoContract, Encode, Decode)]
    pub struct AddPeer {
        pub address: String,
        pub peer_key: PublicKey,
    }
}

const PING_SIZE: usize = 32;

#[derive(Io, Decode, Encode, Debug, Clone)]
enum Message {
    //TODO: introduce other features like block sync, voting and etc.
    Ping(Ping),
    Pong(Ping),
    PendingTx(Transaction),
    AddPeer(PeerId),
    NewPeer(PeerId),
    RemovePeer(PeerId),
}

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, Hash)]
struct Ping {
    payload: Vec<u8>,
    to_peer: PeerId,
    from_peer: PeerId,
}

impl Ping {
    pub fn new(to_peer: PeerId, from_peer: PeerId) -> Ping {
        Ping {
            payload: [0u8; PING_SIZE].to_vec(),
            to_peer,
            from_peer,
        }
    }
}

#[derive(Encode, Decode, PartialEq, Eq, Debug, Clone, Hash)]
struct PeerId {
    listen_address: String,
}

struct PeerState {
    pub peers: HashSet<PeerId>,
    pub sent_pings: HashMap<Ping, Duration>,
    pub listen_address: String,
}

pub struct Peer {
    state: State<PeerState>,
    ping_interval_sec: usize,
    tx_interval_sec: usize,
}

impl Peer {
    pub fn new(listen_address: String, tx_interval_sec: usize, ping_interval_sec: usize) -> Peer {
        Peer {
            state: Arc::new(Mutex::new(PeerState {
                peers: HashSet::new(),
                sent_pings: HashMap::new(),
                listen_address,
            })),
            ping_interval_sec,
            tx_interval_sec,
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        let listen_future = self.listen_and_reconnect().fuse();
        let tx_future = self.start_broadcasting_tx().fuse();
        let ping_future = self.start_ping().fuse();
        pin_mut!(listen_future, tx_future, ping_future);
        select! {
                listen = listen_future => unreachable!(),
                ping = ping_future => ping?,
                tx = tx_future => tx?,
        }
        Ok(())
    }

    pub async fn start_broadcasting_tx(&self) -> Result<(), String> {
        loop {
            async_std::task::sleep(Duration::from_secs(self.tx_interval_sec as u64)).await;
            Self::broadcast_tx(self.state.clone()).await?;
        }
    }

    pub async fn start_ping(&self) -> Result<(), String> {
        loop {
            async_std::task::sleep(Duration::from_secs(self.ping_interval_sec as u64)).await;
            Self::ping_all(self.state.clone()).await?;
        }
    }

    fn current_time() -> Duration {
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Failed to get duration since UNIX_EPOCH.")
    }

    async fn broadcast_tx(state: State<PeerState>) -> Result<(), String> {
        //TODO: check if we have pending tx and then broadcast it
        let tx = Transaction::builder(vec![], Id::new("name", "domain")).build();
        Self::broadcast(state, Message::PendingTx(tx)).await
    }

    async fn listen_and_reconnect(&self) {
        loop {
            if self.listen().await.is_ok() {
                unreachable!()
            }
        }
    }

    async fn listen(&self) -> Result<(), String> {
        async fn handle_request(
            state: State<PeerState>,
            request: Request,
        ) -> Result<Response, String> {
            Peer::handle_message(state, request.payload()).await?;
            Ok(Response::new())
        };

        async fn handle_connection(
            state: State<PeerState>,
            stream: Box<dyn AsyncStream>,
        ) -> Result<(), String> {
            Network::handle_message_async(state, stream, handle_request).await
        };

        let listen_address = self.state.lock().await.listen_address.clone();
        Network::listen(
            self.state.clone(),
            listen_address.as_ref(),
            handle_connection,
        )
        .await
    }

    #[allow(unused)]
    pub async fn start_and_connect(&self, peer_address: &str) -> Result<(), String> {
        let peer_id = PeerId {
            listen_address: peer_address.to_string(),
        };
        self.state.lock().await.peers.insert(peer_id.clone());
        let message = Message::NewPeer(PeerId {
            listen_address: self.state.lock().await.listen_address.clone(),
        });
        Self::send(message, peer_id).await?;
        self.start().await?;
        Ok(())
    }

    async fn send(message: Message, peer_id: PeerId) -> Result<(), String> {
        let _response = Network::send_request_to(
            peer_id.listen_address.as_ref(),
            Request::new("/".to_string(), message.into()),
        )
        .await?;
        Ok(())
    }

    async fn broadcast(state: State<PeerState>, message: Message) -> Result<(), String> {
        let mut send_futures = Vec::new();
        for peer_id in state.lock().await.peers.clone() {
            send_futures.push(Self::send(message.clone(), peer_id));
        }
        let _results = futures::future::join_all(send_futures).await;
        Ok(())
    }

    async fn ping(state: State<PeerState>, peer_id: PeerId) -> Result<(), String> {
        let ping = Ping::new(
            peer_id.clone(),
            PeerId {
                listen_address: state.lock().await.listen_address.clone(),
            },
        );
        state
            .lock()
            .await
            .sent_pings
            .insert(ping.clone(), Peer::current_time());
        Self::send(Message::Ping(ping), peer_id.clone()).await
    }

    async fn ping_all(state: State<PeerState>) -> Result<(), String> {
        for peer_id in state.lock().await.peers.clone() {
            Self::ping(state.clone(), peer_id).await?;
        }
        Ok(())
    }

    async fn handle_message(state: State<PeerState>, bytes: &[u8]) -> Result<(), String> {
        let message: Message = bytes.to_vec().try_into()?;
        match message {
            Message::Ping(ping) => {
                Self::send(Message::Pong(ping.clone()), ping.from_peer).await?;
            }
            Message::Pong(ping) => {
                let sent_pings = &mut state.lock().await.sent_pings;
                if sent_pings.contains_key(&ping) {
                    let sent_time = sent_pings
                        .get(&ping)
                        .expect("Failed to get sent ping entry.");
                    let _rtt = Peer::current_time() - sent_time.to_owned();
                    sent_pings.remove(&ping);
                }
            }
            Message::PendingTx(_tx) => {
                //TODO: handle incoming pending tx
            }
            Message::NewPeer(new_peer_id) => {
                //TODO: use transactions to add a new peer and verify on connection in swarm
                //tell node about other peers
                let mut send_futures = Vec::new();
                for peer_id in state.lock().await.peers.clone() {
                    send_futures.push(Self::send(
                        Message::AddPeer(new_peer_id.clone()),
                        peer_id.clone(),
                    ));
                }
                let _results = futures::future::join_all(send_futures).await;
                //tell other peers about the new node
                Self::broadcast(state.clone(), Message::AddPeer(new_peer_id.clone())).await?;
                //remember new node
                state.lock().await.peers.insert(new_peer_id);
            }
            Message::AddPeer(peer_id) => {
                state.lock().await.peers.insert(peer_id);
            }
            Message::RemovePeer(peer_id) => {
                state.lock().await.peers.remove(&peer_id);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn start_peer(listen_address: &str, connect_to: Option<String>) -> Arc<Peer> {
        use async_std::task;

        let peer = Arc::new(Peer::new(listen_address.to_string(), 10, 15));
        let peer_move = peer.clone();
        task::spawn(async move {
            let _result = match connect_to {
                None => peer_move.start().await,
                Some(connect_to_addr) => {
                    peer_move.start_and_connect(connect_to_addr.as_ref()).await
                }
            };
        });
        peer
    }

    #[async_std::test]
    async fn connect_three_peers() {
        let _peer0 = start_peer("127.0.0.1:7878", None);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let peer1 = start_peer("127.0.0.1:7879", Some("127.0.0.1:7878".to_string()));
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _peer2 = start_peer("127.0.0.1:7880", Some("127.0.0.1:7878".to_string()));
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert_eq!(peer1.state.lock().await.peers.len(), 2);
    }
}