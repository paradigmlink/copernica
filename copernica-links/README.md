# Copernica

RPC (HATEOAS) actions on a universally addressable public key.

Copernica is a transport agnostic overlay, privacy preserving, Information-centric, IP replacement, networking protocol. Just as Bitcoin separates money and state, so Copernica separates your data from -FAANG- state.

This crate contains a variety of different transports. Each transport is complimentary, with compliments sitting on each end a pipe.

[protocol] -> [transport] ----/// \\\ ---- [transport] -> [broker]
or
[broker] -> [transport] ----/// \\\ ---- [transport] -> [broker]

Feel free to add as many transports as you wish: XMPP, morse code, lasers, stenographically embedded into things like SMTP maybe? Go wild.

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Paper

Please read the [paper](https://fractalide.com/fractalide.pdf).

## Authors

* **Stewart Mackenzie** - [sjmackenzie](https://github.com/sjmackenzie)

## License

This project is licensed under the MPLV2 License - see the [LICENSE](LICENSE) file for details

