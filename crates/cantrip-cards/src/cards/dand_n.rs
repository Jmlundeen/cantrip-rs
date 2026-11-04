use cantrip_api::*;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[state_triggered(lands::you_control_no_islands, effects::sacrifice_self,
        "When you control no Islands, sacrifice this creature.")],
    attack_rules: &[lands::attack_rule_defender_controls_island],
    ..oracle::DAND_N
};
