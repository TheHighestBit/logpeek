logpeek
===

Easy to use logging library for Rust with a built-in web based dashboard.

# Logging library implementation

- [ ] Use the API provided by the log crate. This makes logpeek a drop-in replacement for already existing projects.
  - [ ] Implement the Log trait for the logpeek struct.
  - [ ] Ensure compatibility with all of the supported macros.
  - [ ] Look into how error handling should be done exactly (who is responsible for what).
- [ ] Implement the build method, that sets up the logger.
- [ ] Design a Config object with all of the required options.
- [ ] Implement the write to file functionality.
- [ ] Make the write to file functionality run on a dedicated thread.
- [ ] Explore ways of speeding up the write to file process.
- [ ] Implement automatic compression of log files.
- [ ] Explore additional functionality, such as supporting Serde.
- [ ] Support stylized text in terminal (colors and such).
- [ ] Finalize documentation for creates.io


# Web dashboard implementation
TBD