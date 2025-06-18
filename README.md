# Simple Socks4 Server

This project was done for educational purposes. Simple Socks4 Server.

### Usage
```bash
./socks4-server 0.0.0.0 1080
# Bind address 0.0.0.0:1080, Listens for socks4 clients (such as proxychains)
```

### Build
Regular `debug` mode
```bash
cargo build
```
Build with `release` format
```bash
cargo build --release
```
Compile static for `linux x86-64`
```bash
# Prerequisites
rustup target add x86_64-unknown-linux-musl
sudo apt-get install musl-tools
# Build
cargo build --release --target x86_64-unknown-linux-musl
```
Compile static for `linux x86-64`
```bash
# Prerequisites
rustup target add x86_64-pc-windows-gnu
sudo apt-get install mingw-w64
# Build
cargo build --release --target x86_64-pc-windows-gnu
```



Basic Features:
* Listening port and interface given by command line argument
* Logs - Tracing
* Async
* Code docstring

Nice To Have:
* Windows & Linux Support
* More Architectures
    * x86-64
    * x86-32
    * arm
* Production
    * Static
    * Optimized
    * Stripped