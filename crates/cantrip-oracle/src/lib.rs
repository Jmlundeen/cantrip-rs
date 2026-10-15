//! Every card's characteristics (docs/engine-design.md, "Card data: three sources"): generated at build time from content/ (build.rs), never
//! written by hand. A card's behavior adds to its constant here: `CardDef { spell: ..., ..cantrip_oracle::LIGHTNING_BOLT }`.

#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/generated.rs"));
