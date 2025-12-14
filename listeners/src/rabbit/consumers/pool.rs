use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use prost::Message;
use tokio::task::JoinSet;

use crate::lapin::{MessageHandler, RabbitClient};

// Trait for spawning consumer tasks
trait ConsumerSpawner<S>: Send + Sync
where
    S: Send + Sync + 'static,
{
    fn spawn(&self, lapin: Arc<RabbitClient>, state: Arc<S>, join_set: &mut JoinSet<()>);
    fn clone_box(&self) -> Box<dyn ConsumerSpawner<S>>;
}

// Concrete implementation for a typed consumer
struct TypedConsumerSpawner<S, M, H>
where
    S: Send + Sync + 'static,
    M: Message + Default + 'static,
    H: MessageHandler<Arc<S>, M> + Clone + 'static,
{
    queue_name: String,
    handler: H,
    _phantom: PhantomData<(S, M)>,
}

impl<S, M, H> ConsumerSpawner<S> for TypedConsumerSpawner<S, M, H>
where
    S: Send + Sync + 'static,
    M: Message + Default + 'static,
    H: MessageHandler<Arc<S>, M> + Clone + 'static,
{
    fn spawn(&self, lapin: Arc<RabbitClient>, state: Arc<S>, join_set: &mut JoinSet<()>) {
        let queue_name = self.queue_name.clone();
        let handler = self.handler.clone();

        join_set.spawn(async move {
            let _ = lapin.consume_messages(state, queue_name, handler).await;
        });
    }

    fn clone_box(&self) -> Box<dyn ConsumerSpawner<S>> {
        Box::new(TypedConsumerSpawner {
            queue_name: self.queue_name.clone(),
            handler: self.handler.clone(),
            _phantom: PhantomData,
        })
    }
}

// Struct to hold consumers independently from RabbitClient
pub struct Consumers<S>
where
    S: Send + Sync + 'static,
{
    spawners: HashMap<String, Box<dyn ConsumerSpawner<S>>>,
}

impl<S> Clone for Consumers<S>
where
    S: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        let spawners = self
            .spawners
            .iter()
            .map(|(k, v)| (k.clone(), v.clone_box()))
            .collect();
        Self { spawners }
    }
}

impl<S> Consumers<S>
where
    S: Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            spawners: HashMap::new(),
        }
    }

    pub fn add<M, H>(mut self, queue_name: &str, message_handler: H) -> Self
    where
        M: Message + Default + 'static,
        H: MessageHandler<Arc<S>, M> + Clone + 'static,
    {
        let spawner = TypedConsumerSpawner {
            queue_name: queue_name.to_string(),
            handler: message_handler,
            _phantom: PhantomData,
        };

        self.spawners
            .insert(queue_name.to_string(), Box::new(spawner));

        self
    }

    pub fn merge(mut self, other: Consumers<S>) -> Self {
        for (queue_name, spawner) in other.spawners.into_iter() {
            // Only insert if we don't already have this queue
            self.spawners.entry(queue_name).or_insert(spawner);
        }
        self
    }

    pub fn has_consumer(&self, queue_name: &str) -> bool {
        self.spawners.contains_key(queue_name)
    }

    pub fn count(&self) -> usize {
        self.spawners.len()
    }

    // Spawn all consumers
    pub(crate) fn spawn_all(
        self,
        lapin: Arc<RabbitClient>,
        state: Arc<S>,
        join_set: &mut JoinSet<()>,
    ) {
        for (_, spawner) in self.spawners.into_iter() {
            spawner.spawn(Arc::clone(&lapin), Arc::clone(&state), join_set);
        }
    }
}

impl<S> Default for Consumers<S>
where
    S: Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub struct ConsumerPool<S>
where
    S: Send + Sync + 'static,
{
    lapin: Arc<RabbitClient>,
    state: Arc<S>,
    consumers: Consumers<S>,
}

impl<S> ConsumerPool<S>
where
    S: Send + Sync + 'static,
{
    pub fn new(lapin: RabbitClient, state: S, consumers: Consumers<S>) -> Self {
        let lapin = Arc::new(lapin);
        let state = Arc::new(state);

        Self {
            lapin,
            state,
            consumers,
        }
    }

    pub fn is_consuming(&self, queue_name: &str) -> bool {
        self.consumers.has_consumer(queue_name)
    }

    pub fn active_queue_count(&self) -> usize {
        self.consumers.count()
    }

    pub async fn start(self) {
        let mut join_set = JoinSet::new();

        // Spawn all consumers using the Consumers method
        self.consumers
            .spawn_all(self.lapin, self.state, &mut join_set);

        // Wait for all tasks to complete
        join_set.join_all().await;
    }
}
