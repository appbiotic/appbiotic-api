# appbiotic-api

Generally available APIs generated with different flavors by
[Appbiotic API Build](https://github.com/appbiotic/api-build).

- [appbiotic-api](#appbiotic-api)
  - [Flavors](#flavors)

## Flavors

| Language | Flavor | Description | Notes |
| :--- | :--- | :--- | :--- |
| Rust | `prost-serde` | [Protobuf](https://protobuf.dev/) via [`prost`](https://github.com/tokio-rs/prost) and [`tonic`](https://github.com/hyperium/tonic) with [`prost-wkt`](https://github.com/fdeantoni/prost-wkt)'s [`serde`](https://github.com/serde-rs/serde) serialization for JSON support | |
| Rust | `protobuf-v4` | [Protobuf](https://protobuf.dev/) via [`protobuf` v4](https://docs.rs/protobuf/4.30.0-beta1/protobuf) from [Google](https://protobuf.dev/reference/rust/) [[src](https://github.com/protocolbuffers/protobuf/tree/main/rust)] | TBD (Upstream `protobuf` v4 library may have been [implicitly announced as beta in v30.0](https://groups.google.com/g/protobuf/c/G3DZIvytU0o/m/cSStRxfgAQAJ)) |
