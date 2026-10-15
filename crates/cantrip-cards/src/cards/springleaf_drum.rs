use cantrip_api::*;
use cantrip_mechanics::{TAP_AN_UNTAPPED_CREATURE, mana};
use cantrip_oracle as oracle;

/// "{T}, Tap an untapped creature you control: Add one mana of any color." A mana ability (CR 605.1a): the creature is
/// chosen with the activation, like a sacrificed permanent.
pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}, Tap an untapped creature you control: Add one mana of any color.",
        cost: Cost { tap: true, ..TAP_AN_UNTAPPED_CREATURE },
        mana: Some(mana::add_chosen),
        any_color: true,
        adds_any: 1,
        ..ActivatedDef::BLANK
    })],
    ..oracle::SPRINGLEAF_DRUM
};
