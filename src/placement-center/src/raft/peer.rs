
use log::{debug, error, info};
use tokio::sync::mpsc;
use tonic::{Response, Status};
use protocol::placement::placement_center_service_client::PlacementCenterServiceClient;
use protocol::placement::{SendRaftConfChangeReply, SendRaftMessageRequest};

#[derive(Debug, Clone)]
pub struct PeerMessage {
    pub to: String,
    pub data: Vec<u8>,
}

pub struct PeersManager {
    peer_message_recv: mpsc::Receiver<PeerMessage>,
}

impl PeersManager {
    pub fn new(peer_message_recv: mpsc::Receiver<PeerMessage>) -> Self {
        PeersManager {
            peer_message_recv,
        }
    }

    pub async fn start(&mut self) {
        info!(
            "{}",
            "Starts the thread that sends Raft messages to other nodes"
        );

        loop {
            if let Some(data) = self.peer_message_recv.recv().await {
                let addr = data.to;
                let request: SendRaftMessageRequest = SendRaftMessageRequest {
                    message: data.data,
                };
                let mut client = PlacementCenterServiceClient::connect(format!("http://{}", addr))
                    .await
                    .unwrap();

                match client.send_raft_message(request).await {
                    Ok(_) => debug!("Send Raft message to node {} Successful.", addr),
                    Err(e) => error!(
                        "Failed to send data to {}, error message: {}",
                        addr,
                        e.to_string()
                    ),
                }
            }
        }
    }
}