use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::net::ToSocketAddrs;
use std::process::exit;

struct URL {
    scheme: String,
    host: String,
    path: String,
    port: u16,
}

#[derive(Debug)]
struct Response {
    version: String,
    status_code: u16,
    explanation: String,
    headers: HashMap<String, String>,
    body: String,
}

const PROTOCOL_DELIMITER: char = ':';
const PATH_DELIMITER: char = '/';

impl URL {
    fn new(url: &str) -> Self {
        let mut it = url.chars();

        // TODO: use split_once on `:`
        let scheme = it
            .by_ref()
            .take_while(|&c| c != PROTOCOL_DELIMITER)
            .collect::<String>()
            .to_lowercase();

        let host = it
            .by_ref()
            // Some schemes do not have double slash
            .skip_while(|&c| c == PATH_DELIMITER)
            .take_while(|&c| c != PATH_DELIMITER)
            .collect::<_>();

        let mut path = it.collect::<String>();

        path.insert(0, PATH_DELIMITER);

        let port = if scheme == "HTTPS" { 443 } else { 80 };

        URL {
            scheme,
            host,
            path,
            port,
        }
    }

    fn request(self) -> Result<Response, Box<dyn std::error::Error>> {
        let socket = Socket::new(
            /* AF_INET */ Domain::IPV4,
            /* SOCK_STREAM */ Type::STREAM,
            /* IPPROTO_TCP */ Some(Protocol::TCP),
        )?;

        let Ok(mut addresses) = format!("{}:{}", self.host, self.port).to_socket_addrs() else {
            panic!(
                "Failed to resolve, {host}:{port}",
                host = self.host,
                port = self.port
            );
        };

        let Some(address) = addresses.next() else {
            panic!("No address available");
        };

        let Ok(_) = socket.connect(&SockAddr::from(address)) else {
            panic!("Could not connect");
        };

        let request = format!(
            "GET {path} HTTP/1.0\r\nHOST: {host}\r\n\r\n",
            path = self.path,
            host = self.host
        );

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

        let response = String::from_utf8_lossy(&chunks).into_owned();

        let mut response_lines = response.lines();

        println!("Response:");

        let Some(status) = response_lines.next() else {
            panic!("No status in Response");
        };

        let mut status_parts = status.split_whitespace();

        let Some(version) = status_parts.next() else {
            panic!("No version in status");
        };

        let Some(status_code) = status_parts.next() else {
            panic!("No status_code in status");
        };

        let Ok(status_code) = status_code.parse::<u16>() else {
            panic!("Status code is not u16");
        };

        let Some(explanation) = status_parts.next() else {
            panic!("No explanation in status");
        };

        let headers = response_lines
            .by_ref()
            .take_while(|l| !l.is_empty())
            .filter_map(|row| row.split_once(": "))
            .map(|(key, value)| (key.to_lowercase(), value.to_owned()))
            .collect::<HashMap<_, _>>();

        assert!(
            !headers.contains_key("transfer-encoding"),
            "transfer-encoding found"
        );

        assert!(
            !headers.contains_key("content-encoding"),
            "content-encoding found"
        );

        let body = response_lines.collect::<Vec<&str>>().join("\r\n");

        Ok(Response {
            version: version.to_owned(),
            status_code: status_code.to_owned(),
            explanation: explanation.to_owned(),
            headers,
            body,
        })
    }
}

impl Response {
    fn print_body(self) {
        let mut in_tag = false;

        for ch in self.body.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => {
                    print!("{ch}");
                }

                _ => {}
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let Some(url) = &args.get(1) else {
        println!("No target URL was given");

        exit(1)
    };

    let response = URL::new(url).request()?;

    response.print_body();

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
        assert_eq!(result.path, "/index.html");

        let result = URL::new("http://www.example.org/example/index.html");

        assert_eq!(result.scheme, "http");
        assert_eq!(result.host, "www.example.org");
        assert_eq!(result.path, "/example/index.html");

        let result = URL::new("HTTPS://www.example.org/");

        assert_eq!(result.scheme, "https");

        let result = URL::new("HTTPS://www.example.org");

        assert_eq!(result.path, "/");

        let result = URL::new("www.example.org");

        assert_eq!(result.host, "www.example.org");
    }
}
