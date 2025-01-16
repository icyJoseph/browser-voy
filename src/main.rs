use native_tls::TlsConnector;
use std::boxed::Box;
use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::exit;

const PROTOCOL_DELIMITER: char = ':';
const PORT_DELIMITER: char = ':';
const PATH_DELIMITER: char = '/';

#[derive(PartialEq, Debug)]
enum Scheme {
    Https,
    Http,
    File,
    Data,
}

#[allow(unused)]
struct Url {
    scheme: Scheme,
    hostname: String,
    host: String,
    path: String,
    port: u16,
}

#[allow(unused)]
struct Request<'a> {
    method: &'a str,
    url: &'a Url,
}

#[allow(unused)]
#[derive(Debug)]
struct Response {
    version: String,
    status_code: u16,
    explanation: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Scheme {
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

impl<'a> Request<'a> {
    fn new(url: &'a Url, method: &'a str) -> Self {
        Request { method, url }
    }

    fn as_bytes(&self) -> Vec<u8> {
        let request_line = format!(
            "{method} {path} {version}",
            method = self.method,
            path = self.url.path,
            version = "HTTP/1.1"
        );

        let mut request_parts = vec![];

        request_parts.push(request_line);

        let mut headers: HashMap<&str, &str> = HashMap::new();

        headers.insert("Host", &self.url.host);
        headers.insert("Connection", "close");
        headers.insert("User-Agent", "BrowserVoy");

        for (key, value) in headers {
            request_parts.push(format!("{key}: {value}"));
        }

        request_parts.push("\r\n".to_string());

        let request = request_parts.join("\r\n");

        if cfg!(debug_assertions) {
            println!("Request:\n{request}");
        }

        request.as_bytes().to_vec()
    }
}

impl Response {
    fn parse(response: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut response_lines = response.lines();

        if cfg!(debug_assertions) {
            println!("Response:");
        }

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

    fn execute(request: Request) -> String {
        let mut chunks = vec![];

        let Ok(mut socket) = TcpStream::connect(&request.url.host) else {
            panic!("Could not connect");
        };

        if request.url.scheme == Scheme::Https {
            let Ok(connector) = TlsConnector::new() else {
                panic!("Failed to create TLS Connector");
            };

            let Ok(mut tls_socket) = connector.connect(&request.url.hostname, socket) else {
                panic!("Failed to upgrade TLS");
            };

            let _ = tls_socket.write_all(&request.as_bytes());

            let _ = tls_socket.read_to_end(&mut chunks);
        } else {
            let _ = socket.write_all(&request.as_bytes());

            let _ = socket.read_to_end(&mut chunks);
        }
        let response = String::from_utf8_lossy(&chunks).into_owned();

        response
    }

    fn print_body(self) {
        let mut in_tag = false;

        for ch in self.body.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ if !in_tag => {
                    print!("{ch}");
                }

                _ => continue,
            }
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

        Url {
            scheme,
            hostname,
            host,
            path,
            port,
        }
    }

    fn load(self) -> Result<Response, Box<dyn std::error::Error>> {
        let request = Request::new(&self, "GET");

        Response::parse(Response::execute(request))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let Some(url) = &args.get(1) else {
        println!("No target URL was given");

        exit(1)
    };

    let response = Url::new(url).load()?;

    response.print_body();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url() {
        let result = Url::new("https://example.org/index.html");

        assert_eq!(result.scheme, Scheme::Https);
        assert_eq!(result.host, "example.org:443");
        assert_eq!(result.hostname, "example.org");
        assert_eq!(result.path, "/index.html");

        let result = Url::new("http://www.example.org/example/index.html");

        assert_eq!(result.scheme, Scheme::Http);
        assert_eq!(result.host, "www.example.org:80");
        assert_eq!(result.hostname, "www.example.org");
        assert_eq!(result.path, "/example/index.html");

        let result = Url::new("HTTPS://www.example.org/");

        assert_eq!(result.scheme, Scheme::Https);

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
