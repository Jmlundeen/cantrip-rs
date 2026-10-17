# cantrip-rs

A Magic: The Gathering rules engine written in Rust, with a C#/WebAssembly front end built on the Uno Platform.

The goal is the fastest MTG rules engine there is, fast enough for human play and for training state-of-the-art AI players. Four principles guide the work:

- **Exact rules, no exceptions.** We only call a card, mechanic or format supported once it follows the Comprehensive Rules completely. Tests back that claim: every part of every card runs in scenarios that cover its corner cases and interactions.
- **Throughput.** Game state lives in an append-only ledger. Branching it for search costs under 2µs, recording an action is cheap, and so is listing the legal actions. The target is thousands of games per second on a decent desktop CPU with 16 or more cores.
- **Cards scale.** Clear boundaries separate the core rules from each card's code, so a new card touches as little as possible. Coding agents write almost all of the project, so context stays small and local and existing mechanics are easy to find. We aim to keep each new card under three minutes to implement.
- **Hard cards first.** The first cards supported are the MTGO Vintage Cube, the LSV Cube and the PowerLSV Cube. They hold some of the hardest cards ever printed to model, and they are our proof of concept.

## Community

To contribute, follow development or try a cube draft, join our [Discord server](https://discord.gg/Q8K88sncn).

## License

Cantrip is released into the public domain under [the Unlicense](LICENSE).
