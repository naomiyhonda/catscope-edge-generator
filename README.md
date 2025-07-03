# Catscope Edge Generator

## Install Dependencies

```bash
curl https://sh.rustup.rs -sSf | sh
rustup toolchain install 1.84.0
rustup override set 1.84.0
rustup target add wasm32-wasip1
cargo install cargo-deb@2.7.0
```

* do not use the latest `cargo-deb` version as it does not support packaging `wasm32-wasip1` targets into a Debian package for `x86_64`.

## Compile wasm32-wasip1

```bash
cargo build --target wasm32-wasip1
```

## Compile Debian package

```bash
cargo build --target wasm32-wasip1 --release
cargo deb
```

The resulting `.deb` package will be in `./target/debian`.

The wasm will be installed in `/usr/share/catscope/catscope_edge_generator.wasm`.

## Update the Geyser config to reflect the wasm file path:

```json
{
   "libpath":"/usr/lib/libsolana_geyser_plugin_catscope.so",
   "filter": "/usr/share/catscope/catscope_edge_generator.wasm",
   "filter_count": 50,
   "shooter_port_low":15401,
   "shooter_port_high":15416,
   "bot":{
      "core_count": 2
   },
   "log":{
      "level":"warn"
   },
   "net":{
      "listen_url":"0.0.0.0:57451",
      "ring_size": 16384,
      "buffer_size": 10485760,
      "max_write": 16384,
      "store_dir":"/var/share/catscope"
    }
}
```
## Run Tests

```bash
RUST_LOG=debug RUST_BACKTRACE=full CARGO_BUILD_JOBS=2 cargo test -- --nocapture
```