use crate::read_socket::try_read_one_byte;
use crate::server::SubscriberList;
use log::info;
use serde::Deserialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Correct format for message request
#[derive(Debug, Deserialize)]
pub struct Message {
    pub message: String,
}

const ERR_INVALID_JSON: &[u8] = br#"{"error": "received not valid json"}\n"#;

/// Processes requests by publisher. If everything is correct, sends message to all subscribers of
/// the topic. Filters disconnected subscribers while sending messages to them.
///
/// # Arguments
///
/// `socket` - stream to read publisher's messages from
/// `topic_name` - name of the publisher's topic
/// `subscribers` - get current subscribers of the topic
pub async fn listen_publisher(
    socket: Arc<Mutex<TcpStream>>,
    topic_name: String,
    subscribers: Arc<Mutex<SubscriberList>>,
) {
    tokio::spawn(async move {
        'process_client: loop {
            let mut bytes: Vec<u8> = vec![];
            loop {
                match try_read_one_byte(socket.clone()).await {
                    Some(byte) => {
                        if byte == 10 {
                            break;
                        }
                        bytes.push(byte);
                    }
                    None => {
                        info!("Connection to {} publisher was terminated", topic_name);
                        return;
                    }
                }
            }

            let request = String::from_utf8(bytes).unwrap_or_default();
            let json: serde_json::error::Result<Message> = serde_json::from_str(&*request);
            match json {
                Err(_e) => {
                    info!("Invalid message on topic {}: {}", topic_name, request);
                    let mut guard = socket.lock().await;
                    let write_res = guard.write(ERR_INVALID_JSON).await;
                    match write_res {
                        Ok(..) => continue 'process_client,
                        Err(_) => {
                            info!("Connection to {} publisher was terminated", topic_name);
                            return;
                        }
                    }
                }
                _ => {}
            }

            let message = json.unwrap().message;
            info!("New message on topic {}: {}", topic_name, message);
            let mut guard = subscribers.lock().await;
            let topic_subscribers = guard.get(&topic_name);
            drop(guard);

            let mut disconnected = HashSet::new();
            for subscriber in topic_subscribers {
                let mut socket = subscriber.socket.lock().await;
                match socket.write(&request.as_bytes()).await {
                    Ok(_) => {}
                    Err(..) => {
                        disconnected.insert(subscriber.id);
                    }
                }
            }

            let mut guard = subscribers.lock().await;
            guard.remove_disconnected(topic_name.clone(), disconnected);
            drop(guard);
        }
    })
    .await
    .unwrap()
}
