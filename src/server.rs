use crate::server_join_handler::get_new_user_info;
use crate::server_publisher_listener::listen_publisher;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

/// Type of user for new connection
pub enum JoinType {
    /// New connected user is posting to specific topic
    Publisher,
    /// New connected user should get all messages from specific topic
    Subscriber,
    /// New connection was terminated
    Unsuccessful,
}

#[derive(Clone)]
/// Contains socket and unique id for subscriber
pub struct Subscriber {
    /// Unique arc to socket
    pub(crate) socket: Arc<Mutex<TcpStream>>,
    /// Unique id of the user
    pub(crate) id: usize,
}

/// Contains and processes information about topics' subscribers
pub struct SubscriberList {
    topics: HashMap<String, Vec<Subscriber>>,
}

impl SubscriberList {
    /// Create empty struct
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
        }
    }

    /// Add subscriber to topic. Creates topic, if it didn't exist
    ///
    /// # Arguments
    ///
    /// * `topic_name` - Topic, that new subscriber connected to
    /// * `socket` - Socket of the subscriber
    /// * `id` - Unique
    pub fn add(&mut self, topic_name: String, new_subscriber: Subscriber) {
        self.topics
            .entry(topic_name)
            .or_insert(Vec::new())
            .push(new_subscriber);
    }

    /// Get cloned vector of subscribers of the topic
    ///
    /// # Arguments
    ///
    /// * `topic_name` - topic to get the subscribers of
    pub fn get(&mut self, topic_name: &str) -> Vec<Subscriber> {
        return match self.topics.get(topic_name) {
            Some(x) => x.clone(),
            None => Vec::new(),
        };
    }

    /// Delete all subscribers that are known to be disconnected
    /// Deletes topic, if no active subscribers left
    ///
    /// # Arguments
    ///
    /// * `topic_name` - name of the topic to filter subscribers in
    /// * `disconnected` - HashSet of ids of known disconnected users
    pub fn remove_disconnected(&mut self, topic_name: String, disconnected: HashSet<usize>) {
        let filtered = self.get(&topic_name);
        let filtered = filtered
            .into_iter()
            .filter(|subscriber| !disconnected.contains(&subscriber.id))
            .collect();

        if let Some(subscribers) = self.topics.get_mut(&topic_name) {
            *subscribers = filtered;
        }
    }
}

/// Processes requests on given TcpListener. Receives messages from users, gets their role and
/// redirects to other functions
///
/// # Arguments
///
/// * `server` - main TcpListener for all connections from users
pub async fn process_messages(server: TcpListener) {
    let subscribers = Arc::new(Mutex::new(SubscriberList::new()));

    let mut id: usize = 0;
    loop {
        let (socket, ip) = server.accept().await.unwrap();
        let subscribers = subscribers.clone();
        tokio::spawn(async move {
            let socket = Arc::new(Mutex::new(socket));
            let (user_type, topic_name) = get_new_user_info(socket.clone(), ip).await;
            match user_type {
                JoinType::Unsuccessful => return,
                JoinType::Publisher => {
                    listen_publisher(socket, topic_name, subscribers.clone()).await;
                }
                JoinType::Subscriber => {
                    let mut subscribers = subscribers.lock().await;
                    subscribers.add(topic_name, Subscriber { socket, id });
                }
            }
        });
        id += 1;
    }
}
