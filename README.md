A postgres client without any third-party dependencies/libraries.

## Roadmap

- [x] Send `StartupMessage` request
- [x] Parse `StartupMessage` response
- [ ] Send `SimpleQuery`(SELECT 1::char;)  request
- [ ] Async support

## what I learn from this project

- little/bigger/naive endian, compile-time env CARGO_CFG_TARGET_ENDIAN
