# Browser Voy

A basic web browser.

## Why

I ran into a LinkedIn post, where the question, `What happens when you type a URL into a browser?`, was portrayed as
the ultimate way to evaluate a developer's profiency.

So I decided to learn how to make a browser.

## What

This is written in Rust. I initially tried to use system calls and such, but
a whole bunch of those are not very nice to work with, from Rust, and things
such as TLS upgrade, became harder than they should've been.

### Features

- [x] HTTP 1.0
- [x] TLS upgrade for HTTPS scheme
- [x] Self made URL parser

For example:

- Load data

```console
cargo run -- "data:text/html,&copy;&apos;&ndash;&nbsp;&lt;&gt;"

©'– <>
```

- Load a site

```console
 cargo run -- https://browser.engineering/examples/example1-simple.html


Request:
GET /examples/example1-simple.html HTTP/1.1
Connection: close
User-Agent: BrowserVoy
Host: browser.engineering:443


Response:


    This is a simple
    web page with some
    text in it.

```

### Planned

- [ ] view-source
- [ ] caching
- [ ] compression
- [ ] redirects
- [ ] keep-alive
