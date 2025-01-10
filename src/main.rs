struct URL {
    scheme: String,
    host: String,
    path: String,
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

        URL { scheme, host, path }
    }
}

fn main() {
    println!("Hello, world!");
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
