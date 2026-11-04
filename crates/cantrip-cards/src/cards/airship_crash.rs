use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_ENCHANTMENT_OR_CREATURE_WITH_FLYING], effects::destroy_first_target),
    abilities: &[triggers::cycling(mana("{2}"), "Cycling {2} ({2}, Discard this card: Draw a card.)")],
    ..oracle::AIRSHIP_CRASH
};
