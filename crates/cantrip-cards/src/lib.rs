//! The spike's cards (docs/engine-design.md, plan step 2): the basic lands and nineteen cards whose 49
//! scenarios exercise priority, the stack, combat, targeting, triggers, copy effects, alternative costs, "unless"
//! costs and effects that ask mid-resolution.
//!
//! One file per card. Characteristics are written here until the Oracle generator (`cantrip-oracle`) exists.

use cantrip_api::CardDef;

include!(concat!(env!("OUT_DIR"), "/cards.rs"));
