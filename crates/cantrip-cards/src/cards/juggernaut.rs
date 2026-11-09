use cantrip_api::*;
use cantrip_mechanics::{combat, statics};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            statics::itself,
            statics::attacks_each_combat_if_able,
            "This creature attacks each combat if able.",
        ),
    ],
    block_rules: &[combat::cant_be_blocked_by_walls],
    ..oracle::JUGGERNAUT
};
