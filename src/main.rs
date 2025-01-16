use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::net::{TcpStream, ToSocketAddrs};
use std::process::exit;

struct Url {
    scheme: String,
    hostname: String,
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
const PORT_DELIMITER: char = ':';
const PATH_DELIMITER: char = '/';

#[derive(PartialEq)]
enum Scheme {
    Https,
    Http,
    File,
    Data,
}

impl Scheme {
    fn value(self) -> String {
        let value = match self {
            Scheme::Https => "https",
            Scheme::Http => "http",
            Scheme::File => "file",
            Scheme::Data => "data",
        };

        value.to_string()
    }

    fn extract(url: &str) -> (Self, &str) {
        let (scheme, rest) = match url.split_once(PROTOCOL_DELIMITER) {
            None => ("", url),
            Some((scheme, rest)) => (scheme, rest),
        };

        let scheme = scheme.to_lowercase();

        match scheme.as_str() {
            "" | "https" => (Scheme::Https, rest),
            "http" => (Scheme::Http, rest),
            "file" => (Scheme::File, rest),
            "data" => (Scheme::Data, rest),
            _ => (Scheme::Https, url),
        }
    }
}

impl Url {
    fn new(url: &str) -> Self {
        let (scheme, rest) = Scheme::extract(url);

        let mut it = rest.chars();

        let host = it
            .by_ref()
            // Some schemes do not have double slash
            .skip_while(|&c| c == PATH_DELIMITER)
            .take_while(|&c| c != PATH_DELIMITER)
            .collect::<String>();

        let (hostname, port) = match host.split_once(PORT_DELIMITER) {
            None => (host, if scheme == Scheme::Https { 443 } else { 80 }),
            Some((hostname, port)) => {
                let Some(port) = port.parse::<u16>().ok() else {
                    panic!("Unexpected port {port}");
                };

                (hostname.to_string(), port)
            }
        };

        let host = format!("{hostname}:{port}");

        let mut path = it.collect::<String>();

        path.insert(0, PATH_DELIMITER);

        let scheme = scheme.value();

        Url {
            scheme,
            hostname,
            host,
            path,
            port,
        }
    }

    fn request(self) -> Result<Response, Box<dyn std::error::Error>> {
        let Ok(mut addresses) = self.host.to_socket_addrs() else {
            panic!("Failed to resolve, {host}", host = self.host,);
        };

        let Some(address) = addresses.next() else {
            panic!("No address available");
        };

        let Ok(mut socket) = TcpStream::connect(address) else {
            panic!("Could not connect");
        };

        let request = format!(
            "GET {path} HTTP/1.0\r\nHOST: {host}\r\n\r\n",
            path = self.path,
            host = self.host
        );

        let _ = socket.write(request.as_bytes());

        println!("Request:\n{request}");

        let mut chunks = vec![];

        loop {
            let mut buffer = [0; 1 << 8 /* 256 */];

            let Ok(received) = socket.read(&mut buffer[..]) else {
                panic!("Failed to receive buffer");
            };

            if received == 0 {
                break;
            }

            // append also clears buffer
            chunks.extend_from_slice(&mut buffer);
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

    let response = Url::new(url).request()?;

    response.print_body();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url() {
        let result = Url::new("https://example.org/index.html");

        assert_eq!(result.scheme, "https");
        assert_eq!(result.host, "example.org:443");
        assert_eq!(result.hostname, "example.org");
        assert_eq!(result.path, "/index.html");

        let result = Url::new("http://www.example.org/example/index.html");

        assert_eq!(result.scheme, "http");
        assert_eq!(result.host, "www.example.org:80");
        assert_eq!(result.hostname, "www.example.org");
        assert_eq!(result.path, "/example/index.html");

        let result = Url::new("HTTPS://www.example.org/");

        assert_eq!(result.scheme, "https");

        let result = Url::new("HTTPS://www.example.org");

        assert_eq!(result.path, "/");

        let result = Url::new("www.example.org");

        assert_eq!(result.hostname, "www.example.org");

        let result = Url::new("www.example.org:8080");

        assert_eq!(result.hostname, "www.example.org");
        assert_eq!(result.host, "www.example.org:8080");
        assert_eq!(result.port, 8080);
    }
}
