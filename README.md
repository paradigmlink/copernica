# Copernica

Copernica is privacy preserving Information-centric networking protocol designed to operate over UDP and Radio Frequency.

## Getting Started

Install `rustup`.

## Building

Run nix-shell to make dependencies available in the environment.
`$ nix-shell`

### `copernica`

`$ rustup run nightly cargo build --release --bin copernica`

### `ccli`

- Plug in your STLinkv2 connected to your Copernica Sentinel Hardware Dongle

`$ rustup run nightly cargo run --bin ccli -- --chip STM32F103TB --elf copernica-sentinel/target/thumbv7m-none-eabi/debug/copernica-sentinel`

### `copernica-sentinel`

- Plug in your STLinkv2 connected to your Copernica Sentinel Hardware Dongle

`$ rustup run nightly cargo install probe-run`
`$ cd copernica-sentinel`
`$ rustup run nightly cargo run`

## Running regressions

`$ rustup run nightly cargo run --bin copernica-tests`

## Running the tests

`$ rustup run nightly cargo test`

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Paper

Please read the [paper](https://fractalide.com/fractalide.pdf).

## Authors

* **Stewart Mackenzie** - [sjmackenzie](https://github.com/sjmackenzie)

## License

This project is licensed under the MPLV2 License - see the [LICENSE](LICENSE) file for details

