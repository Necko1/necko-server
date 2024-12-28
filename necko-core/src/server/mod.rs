use tokio::sync::RwLock;
use crate::server::cached::CachedStatus;

pub mod status;
mod cached;

pub struct Server {
    pub cached_status: RwLock<CachedStatus>
}

impl Server {
    pub fn new() -> Self {
        Server {
            cached_status: RwLock::new(CachedStatus::new())
        }
    }
}

