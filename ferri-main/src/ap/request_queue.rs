use std::sync::mpsc;
use std::thread;
use tracing::{debug, info, span, Level};

#[derive(Debug)]
pub enum QueueMessage {
    Heartbeat
}

pub struct RequestQueue {
    name: &'static str,
    send: mpsc::Sender<QueueMessage>,
    recv: mpsc::Receiver<QueueMessage>
}

#[derive(Clone)]
pub struct QueueHandle {
    send: mpsc::Sender<QueueMessage>
}

impl QueueHandle {
    pub fn send(&self, msg: QueueMessage) {
        self.send.send(msg).unwrap();
    }
}

impl RequestQueue {
    pub fn new(name: &'static str) -> Self {
        let (send, recv) = mpsc::channel();
        Self {
            name,
            send,
            recv
        }
    }

    pub fn spawn(self) -> QueueHandle {
        info!("starting up queue '{}'", self.name);
        
        thread::spawn(move || {
            info!("queue '{}' up", self.name);
            let recv = self.recv;
            
            while let Ok(req) = recv.recv() {
                let s = span!(Level::INFO, "queue", queue_name = self.name);
                let _enter = s.enter();
                
                info!(?req, "got a message into the queue");
                
                drop(_enter);
            }
        });
        
        QueueHandle { send: self.send }
    }
}
