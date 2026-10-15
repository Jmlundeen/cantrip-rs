//! Bestow [cost] (CR 702.103): "If you cast this card for its bestow cost, it's an Aura enchantment spell with enchant
//! creature. It becomes a creature again if it becomes unattached." Detective's Phoenix and Springheart Nantuko print it.
//!
//! The bestow field of the card (`bestow: Some(&BESTOWED)`) is the core's. A card with bestow builds `BESTOWED` with
//! `bestowed`, giving what is different about the bestowed spell: its cost, and anything else the card says.

use crate::targets;
use cantrip_api::*;

/// The bestowed form of `card` (CR 702.103b, 205.1a): an Aura enchantment with "enchant creature", not a creature, with
/// no power or toughness.
pub const fn bestowed(card: CardDef) -> CardDef {
    CardDef {
        types: TypeSet::of(&[CardType::Enchantment]),
        subtypes: SubtypeSet::of(&[Subtype::Aura]),
        power: None,
        toughness: None,
        enchant: Some(targets::CREATURE),
        ..card
    }
}
