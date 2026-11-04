use cantrip_api::*;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        state_triggered(lands::you_control_no_islands, effects::sacrifice_self, "When you control no Islands, sacrifice this creature."),
        ability(cost("{U}"), effects::this_gains_flying, "{U}: This creature gains flying until end of turn."),
    ],
    attack_rules: &[lands::attack_rule_defender_controls::<{ lands::sub(Subtype::Island) }>],
    ..oracle::VODALIAN_KNIGHTS
};
