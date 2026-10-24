use cantrip_api::*;
use cantrip_mechanics::statics::{cant_be_countered, your_spells_of_types};
use cantrip_oracle as oracle;

/// "Creature spells you control": a creature spell on the stack controlled by the player who controls this permanent.
const CREATURE: u16 = TypeSet::of(&[CardType::Creature]).0;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, your_spells_of_types::<CREATURE>, cant_be_countered, "Creature spells you control can't be countered.").everywhere()],
    ..oracle::PROWLING_SERPOPARD
};
