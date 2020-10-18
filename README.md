A postgres client without any third-party dependencies/libraries.

## Roadmap

- [x] Send `StartupMessage` request
- [x] Parse `StartupMessage` response
- [x] Send `SimpleQuery`(SELECT 1::char;) request
- [x] Parse `SimpleQuery`(SELECT 1::char;) response
- [ ] Async support

## postgres protocol message

### client StartupMessage

### server 

## what I learn from this project

- little/bigger/naive endian, compile-time env CARGO_CFG_TARGET_ENDIAN
