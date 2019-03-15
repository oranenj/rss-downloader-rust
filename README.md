# Stupid simple RSS downloader

See config.toml.example for an example downloader

## How it works

It will download the RSS, parse it for anything whose title matches any of the regexps in matchers, hash its contents with sha2 (to sanitize the filename) and save it in `download_dir`. If this succeeds, it will place a marker file based on the downloaded RSS item's GUID (Again, hashed with sha2 for safety) in `cache_dir`

## Building

```
cargo build --release
```
You can also try building a static binary with `--target x86_64-unknown-linux-musl` (requires musl-gcc)
