# logpeek
`logpeek` is a logger implementation for the `log` crate, which focuses on reliability and simplicity.
It is meant to integrate seamlessly with logpeek-server.


- **Remote monitoring with logpeek-server.** Logpeek is designed to work with [logpeek-server] by default, a web-based log tail explorer.

- **A drop-in replacement for existing logger implementations.** If already using the `log` crate and it's macros,
simply replace your existing logger with `logpeek` and you're good to go.

- **Ease of use.** The logger is configured via a `Config` struct, although
for most use cases, the default configuration will suffice.  

[logpeek-server]: https://github.com/TheHighestBit/logpeek-server

## Usage
Use `cargo add logpeek log` or include them in your `Cargo.toml`.

Initializing the logger is as simple as

```rust
use logpeek;
use log::error;

// See the documentation for the config module for more options
let config = logpeek::config::Config {
    logging_mode: logpeek::config::LoggingMode::FileAndConsole,
    datetime_format: logpeek::config::DateTimeFormat::Custom("[hour]:[minute]:[second]:[subsecond][offset_hour sign:mandatory]"), // Logpeek-server requires the UTC offset to be present. 
   ..Default::default()
};

logpeek::init(config).unwrap(); // For the default config use logpeek::init(Default::default()).unwrap();

error!("This is a test error!");
```
