use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// "{T}, Pay 1 life: Add one mana of any color." A mana ability (CR 605.1a) whose cost includes life (CR 119.4).
pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{T}, Pay 1 life: Add one mana of any color.",
        cost: Cost { tap: true, life: 1, ..Cost::NONE },
        mana: Some(mana::add_chosen),
        any_color: true,
        adds_any: 1,
        ..ActivatedDef::BLANK
    })],
    ..oracle::MANA_CONFLUENCE
};
