# Browser Voy

A basic, yet complete, web browser.

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
