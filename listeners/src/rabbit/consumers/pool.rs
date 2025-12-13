use std::sync::{Arc, RwLock};

use prost::Message;
use tokio::task::JoinSet;

use crate::{
    lapin::{MessageHandler, RabbitClient},
    rabbit::consumers::AppState,
};

pub struct ConsumerPool {
    pool: Arc<RwLock<JoinSet<()>>>,
    lapin: Arc<RabbitClient>,
    app_state: Arc<AppState>,
}

impl ConsumerPool {
    pub fn new(lapin: RabbitClient, app_state: AppState) -> Self {
        let pool = Arc::new(RwLock::new(JoinSet::<()>::new()));
        let lapin = Arc::new(lapin);
        let app_state = Arc::new(app_state);
        Self {
            pool,
            lapin,
            app_state,
        }
    }

    pub fn consume<M, E>(self, queue_name: String, message_handler: MessageHandler<M, E>) -> Self
    where
        M: Message + Default + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        let lapin = self.lapin.clone();
        let app_state = self.app_state.clone();
        let mut pool = self
            .pool
            .write()
            .expect("Error occured while writing to the consumers pool");
        pool.spawn(async move {
            let _ = lapin
                .consume_messages((*app_state).clone(), queue_name, message_handler)
                .await;
        });
        Self {
            pool: Arc::clone(&self.pool),
            lapin: self.lapin,
            app_state: self.app_state,
        }
    }

    pub async fn start(self) {
        let pool = Arc::try_unwrap(self.pool)
            .expect("Pool still has multiple references")
            .into_inner()
            .expect("Lock poisoned");

        pool.join_all().await;
    }
}
