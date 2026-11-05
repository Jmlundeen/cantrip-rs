use cantrip_api::*;
use cantrip_mechanics::{combat, cumulative_upkeep};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        cumulative_upkeep!(
            cumulative_upkeep::Payment::Mana(mana("{1}")),
            "Cumulative upkeep"
        ),
    ],
    block_rules: &[combat::cant_be_blocked_except_by_walls],
    ..oracle::FLOW_OF_MAGGOTS
};
