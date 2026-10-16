use cantrip_api::*;
use cantrip_oracle as oracle;

/// "{T}, Pay 1 life: Add one mana of any color." A mana ability (CR 605.1a) whose cost includes life (CR 119.4).
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(TAP.and(life(1)), add_any_color(1), "{T}, Pay 1 life: Add one mana of any color.")],
    ..oracle::MANA_CONFLUENCE
};
