use cantrip_api::*;
use cantrip_mechanics::TAP_AN_UNTAPPED_CREATURE;
use cantrip_oracle as oracle;

/// "{T}, Tap an untapped creature you control: Add one mana of any color." A mana ability (CR 605.1a): the creature is
/// chosen with the activation, like a sacrificed permanent.
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP.and(TAP_AN_UNTAPPED_CREATURE), add_any_color(1), "{T}, Tap an untapped creature you control: Add one mana of any color.")],
    ..oracle::SPRINGLEAF_DRUM
};
