# Copernica

RPC (HATEOAS) actions on a universally addressable public key.

Copernica is a transport agnostic overlay, privacy preserving, Information-centric, IP replacement, networking protocol. Just as Bitcoin separates money and state, so Copernica separates your data from -FAANG- state.

This crate contains that which is sent over the wires. Packets!

Link Packets are the packets that wrap the NarrowWaistPacket and they go over the links (transports)
Inter Link Packets wrap Link Packets and are used exclusively for deciding when and where to forward on a packet.

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## Paper

Please read the [paper](https://fractalide.com/fractalide.pdf).

## Authors

* **Stewart Mackenzie** - [sjmackenzie](https://github.com/sjmackenzie)

## License

This project is licensed under the MPLV2 License - see the [LICENSE](LICENSE) file for details

