use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
};

#[derive(Debug, Clone)]
pub struct AIClient {
    pub addr: SocketAddr,
}

pub enum AiError {
    ConnectionFailed,
    QueryFailed,
    ReadFailed,
    Utf8ConvertFailed,
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
