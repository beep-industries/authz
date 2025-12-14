use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use prost::Message;
use tokio::task::JoinSet;

use crate::lapin::{MessageHandler, RabbitClient};

// Trait for spawning consumer tasks
trait ConsumerSpawner<S>: Send + Sync
where
    S: Send + Sync + 'static,
{
    fn spawn(&self, lapin: Arc<RabbitClient>, state: Arc<S>, join_set: &mut JoinSet<()>);
}

// Concrete implementation for a typed consumer
struct TypedConsumerSpawner<S, M>
where
    S: Send + Sync + 'static,
    M: Message + Default + 'static,
{
    queue_name: String,
    handler: MessageHandler<Arc<S>, M>,
}

impl<S, M> ConsumerSpawner<S> for TypedConsumerSpawner<S, M>
where
    S: Send + Sync + 'static,
    M: Message + Default + 'static,
{
    fn spawn(&self, lapin: Arc<RabbitClient>, state: Arc<S>, join_set: &mut JoinSet<()>) {
        let queue_name = self.queue_name.clone();
        let handler = self.handler;

        join_set.spawn(async move {
            let _ = lapin.consume_messages(state, queue_name, handler).await;
        });
    }
}

pub struct ConsumerPool<S>
where
    S: Send + Sync,
{
    lapin: Arc<RabbitClient>,
    state: Arc<S>,
    consumers: Arc<RwLock<HashMap<String, Box<dyn ConsumerSpawner<S>>>>>,
}

impl<S> ConsumerPool<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(lapin: RabbitClient, state: S) -> Self {
        let lapin = Arc::new(lapin);
        let state = Arc::new(state);
        let consumers = Arc::new(RwLock::new(HashMap::new()));
        Self {
            lapin,
            state,
            consumers,
        }
    }

    pub fn consume<M>(self, queue_name: String, message_handler: MessageHandler<Arc<S>, M>) -> Self
    where
        M: Message + Default + 'static,
    {
        let spawner = TypedConsumerSpawner {
            queue_name: queue_name.clone(),
            handler: message_handler,
        };

        let mut consumers = self.consumers.write().expect("Failed to lock consumers");

        consumers.insert(queue_name, Box::new(spawner));

        drop(consumers);
        self
    }

    pub fn merge(self, other: ConsumerPool<S>) -> Self {
        // Extract other's consumers
        let other_consumers = Arc::try_unwrap(other.consumers)
            .unwrap_or_else(|_| panic!("Other consumers still has multiple references"))
            .into_inner()
            .expect("Lock poisoned");

        let mut self_consumers = self
            .consumers
            .write()
            .expect("Failed to lock self consumers");

        // Move all consumers from other into self
        for (queue_name, spawner) in other_consumers.into_iter() {
            // Only insert if we don't already have this queue
            self_consumers.entry(queue_name).or_insert(spawner);
        }

        drop(self_consumers);

        self
    }

    pub fn is_consuming(&self, queue_name: &str) -> bool {
        self.consumers
            .read()
            .expect("Failed to lock consumers")
            .contains_key(queue_name)
    }

    pub fn active_queue_count(&self) -> usize {
        self.consumers
            .read()
            .expect("Failed to lock consumers")
            .len()
    }

    pub async fn start(self) {
        let mut join_set = JoinSet::new();

        let consumers = Arc::try_unwrap(self.consumers)
            .unwrap_or_else(|_| panic!("Consumers still has multiple references"))
            .into_inner()
            .expect("Lock poisoned");

        // Spawn all consumers
        for (_, spawner) in consumers.into_iter() {
            spawner.spawn(
                Arc::clone(&self.lapin),
                Arc::clone(&self.state),
                &mut join_set,
            );
        }

        // Wait for all tasks to complete
        join_set.join_all().await;
    }
}
