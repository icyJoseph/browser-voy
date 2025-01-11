use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::boxed::Box;
use std::net::ToSocketAddrs;

struct URL {
    scheme: String,
    host: String,
    path: String,
    port: u16,
}

const PROTOCOL_DELIMITER: char = ':';
const PATH_DELIMITER: char = '/';

impl URL {
    fn new(url: &str) -> Self {
        let mut it = url.chars();

        let scheme = it
            .by_ref()
            .take_while(|&c| c != PROTOCOL_DELIMITER)
            .collect::<String>()
            .to_lowercase();

        let host = it
            .by_ref()
            .skip_while(|&c| c == PATH_DELIMITER)
            .take_while(|&c| c != PATH_DELIMITER)
            .collect::<_>();

        let mut path = it.collect::<String>();

        path = if path.is_empty() {
            "/".to_string()
        } else {
            path
        };

        let port = if scheme == "HTTPS" { 443 } else { 80 };

        URL {
            scheme,
            host,
            path,
            port,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let socket = Socket::new(
        /* AF_INET */ Domain::IPV4,
        /* SOCK_STREAM */ Type::STREAM,
        /* IPPROTO_TCP */ Some(Protocol::TCP),
    )?;

    let url = URL::new("http://example.edu");

    let Ok(mut addresses) = format!("{}:{}", url.host, url.port).to_socket_addrs() else {
        panic!(
            "Failed to resolve, {host}:{port}",
            host = url.host,
            port = url.port
        );
    };

    let Some(address) = addresses.next() else {
        panic!("No address available");
    };

    let Ok(_) = socket.connect(&SockAddr::from(address)) else {
        panic!("Could not connect");
    };

    let request = "GET / HTTP/1.0\r\nHOST: example.edu\r\n\r\n";

    println!("Request:\n{request}");

    let Ok(_) = socket.send(request.as_bytes()) else {
        panic!("Failed to send request");
    };

    let mut chunks = vec![];
    // let mut buffer = Vec::with_capacity(1 << 16);
    let mut buffer = Vec::with_capacity(1 << 8 /* 256 */);

    loop {
        let Ok(received) = socket.recv(buffer.spare_capacity_mut()) else {
            panic!("Failed to receive buffer");
        };

        if received == 0 {
            break;
        }

        unsafe { buffer.set_len(received) }

        // append also clears buffer
        chunks.append(&mut buffer);
    }

    println!("Response:\n{s}", s = String::from_utf8_lossy(&chunks));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url() {
        let result = URL::new("https://example.org/index.html");

        assert_eq!(result.scheme, "https");
        assert_eq!(result.host, "example.org");
        assert_eq!(result.path, "index.html");

        let result = URL::new("http://www.example.org/example/index.html");

        assert_eq!(result.scheme, "http");
        assert_eq!(result.host, "www.example.org");
        assert_eq!(result.path, "example/index.html");

        let result = URL::new("HTTPS://www.example.org/");

        assert_eq!(result.scheme, "https");

        let result = URL::new("HTTPS://www.example.org");

        assert_eq!(result.path, "/");
    }
}
