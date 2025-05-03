use tokio::sync::mpsc;
use tracing::{info, span, Instrument, Level};

use crate::ap::http::HttpClient;
use crate::config::Config;
use crate::federation::inbox::handle_inbox_request;
use crate::federation::outbox::handle_outbox_request;

use super::inbox::InboxRequest;
use super::outbox::OutboxRequest;

#[derive(Debug)]
pub enum QueueMessage {
    Heartbeat,
    Inbound(InboxRequest),
    Outbound(OutboxRequest),
}

pub struct RequestQueue {
    name: &'static str,
    send: mpsc::Sender<QueueMessage>,
    recv: mpsc::Receiver<QueueMessage>,
}

#[derive(Clone, Debug)]
pub struct QueueHandle {
    send: mpsc::Sender<QueueMessage>,
}

impl QueueHandle {
    pub async fn send(&self, msg: QueueMessage) {
        self.send.send(msg).await.unwrap();
    }
}

impl RequestQueue {
    pub fn new(name: &'static str) -> Self {
        let (send, recv) = mpsc::channel(1024);
        Self { name, send, recv }
    }

    pub fn spawn(self, config: Config) -> QueueHandle {
        info!("starting up queue '{}'", self.name);
        let span = span!(Level::INFO, "queue", queue_name = self.name);
        
        let fut = async move {
            info!("using config {:#?}, queue is up", config);
            let mut recv = self.recv;
            let http = HttpClient::new();

            while let Some(req) = recv.recv().await {
                info!(?req, "got a message into the queue");
                
                match req {
                    QueueMessage::Heartbeat => {
                        info!("heartbeat on queue");
                    },
                    QueueMessage::Inbound(inbox_request) => {
                        handle_inbox_request(inbox_request, &http).await;
                    },
                    QueueMessage::Outbound(outbox_request) => {
                        handle_outbox_request(outbox_request, &http).await;
                    },
                }
            }
        }.instrument(span);
        
        tokio::spawn(fut);

        QueueHandle { send: self.send }
    }
}
