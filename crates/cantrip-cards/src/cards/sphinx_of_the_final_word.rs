use cantrip_api::*;
use cantrip_mechanics::statics::{cant_be_countered, your_spells_of_types};
use cantrip_oracle as oracle;

/// "Instant and sorcery spells you control": an instant spell or a sorcery spell on the stack controlled by the player who
/// controls this permanent.
const INSTANT_OR_SORCERY: u16 = (1 << CardType::Instant as u16) | (1 << CardType::Sorcery as u16);

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, your_spells_of_types::<INSTANT_OR_SORCERY>, cant_be_countered, "Instant and sorcery spells you control can't be countered.").everywhere()],
    ..oracle::SPHINX_OF_THE_FINAL_WORD
};
