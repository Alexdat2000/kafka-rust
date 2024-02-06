use crate::read_socket::try_read_one_byte;
use crate::server::JoinType;
use log::info;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

const ERR_MESSAGE: &[u8] = br#"{"error": "Incorrect join message. Try --help"}\n"#;

/// Possible methods when joining
#[derive(Serialize, Deserialize, Debug)]
pub enum JoinMethod {
    #[serde(rename = "subscribe")]
    Subscriber,

    #[serde(rename = "publish")]
    Publisher,
}

/// Correct format for join request
#[derive(Debug, Deserialize)]
pub struct JoinMessage {
    /// Type of connected user
    pub method: JoinMethod,
    /// Topic name
    pub topic: String,
}

/// Processes user's first message, where they should specify their role and topic
/// Returns JoinType and topic name (if identification was successful)
///
/// # Arguments
///
/// `socket` - stream to read new user's messages from
/// `ip` - user's ip for logs
pub async fn get_new_user_info(
    socket: Arc<Mutex<TcpStream>>,
    ip: SocketAddr,
) -> (JoinType, String) {
    tokio::spawn(async move {
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
                    info!("New connection was terminated");
                    return (JoinType::Unsuccessful, String::new());
                }
            }
        }
        let request = String::from_utf8(bytes).unwrap_or_default();
        let mut guard = socket.lock().await;
        let json: serde_json::error::Result<JoinMessage> = serde_json::from_str(&*request);
        return match json {
            Err(_e) => {
                let _ = guard.write(ERR_MESSAGE).await;
                info!("Received invalid join message");
                (JoinType::Unsuccessful, String::new())
            }
            Ok(json) => {
                match json.method {
                    JoinMethod::Subscriber => {
                        info!(
                            "For topic {} connected subscriber with ip {}",
                            json.topic, ip
                        );
                        (JoinType::Subscriber, json.topic)
                    }
                    JoinMethod::Publisher => {
                        info!(
                            "For topic {} connected publisher with ip {}",
                            json.topic, ip
                        );
                        (JoinType::Publisher, json.topic)
                    }
                }
            }
        }
    })
    .await
    .unwrap()
}
