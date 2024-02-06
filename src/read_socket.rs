use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Read 1 byte from socket, returns option - u8 or None if connection was terminated
///
/// # Arguments
///
/// * `socket` - Stream to read a byte from
pub async fn try_read_one_byte(socket: Arc<Mutex<TcpStream>>) -> Option<u8> {
    let mut guard = socket.lock().await;
    let mut buff = vec![0; 1];
    match guard.read(&mut buff).await {
        Ok(_) => Some(buff.into_iter().take(1).collect::<Vec<_>>()[0]),
        Err(_) => None,
    }
}
