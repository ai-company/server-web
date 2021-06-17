use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

pub enum AiError {
    ConnectionFailed,
    QueryFailed,
    ReadFailed,
    Utf8ConvertFailed,
}

/// Connector for AI model server
///
/// Flow:
/// * try connect to model server
///     * if not connected:
///         * return ConnectionFailed
/// * try send full text query to server
///     * if not sent:
///         * return QueryFailed
/// * try read response (expected json containing full text diff)
///     * if not received:
///         * return ReadFailed
/// * try parse response (expected UTF-8)
///     * if invalid:
///         * return Utf8ConvertFailed
/// * return response
///
/// TODO: implement stage completion messages and handling
///
/// TODO: implement pool (model side?)
#[derive(Debug, Clone)]
pub struct AIClient {
    pub addr: SocketAddr,
}

impl AIClient {
    pub fn new<T: Into<SocketAddr>>(addr: T) -> Self {
        AIClient { addr: addr.into() }
    }

    pub fn request(&self, query: &str) -> Result<String, AiError> {
        let mut connection = match TcpStream::connect(self.addr) {
            Ok(connection) => connection,
            Err(_) => return Err(AiError::ConnectionFailed),
        };

        if let Err(_) = connection.write(query.as_bytes()) {
            return Err(AiError::QueryFailed);
        };

        let mut response = Vec::new();
        if let Err(_) = connection.read_to_end(&mut response) {
            return Err(AiError::ReadFailed);
        }

        match String::from_utf8(response) {
            Ok(result) => Ok(result),
            Err(_) => Err(AiError::Utf8ConvertFailed),
        }
    }
}
