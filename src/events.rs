use tokio::sync::broadcast::{self};
use uuid::Uuid;

use crate::model::auth::UserStatus;

#[derive(Clone)]
pub struct EventChannel {
    sender: broadcast::Sender<Event>,
}

impl EventChannel {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: Event) -> Result<usize, broadcast::error::SendError<Event>> {
        self.sender.send(event)
    }
}

#[derive(Clone, Debug)]
pub enum Event {
    UserStatusUpdate {
        user_id: Uuid,
        new_status: UserStatus,
    },
}
