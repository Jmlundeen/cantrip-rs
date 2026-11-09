use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}").and(TAP), prevention::prevent_damage_by_target_then_toughness_equal_to_mana_value,
        "{2}, {T}: Prevent all damage that would be dealt this turn by target creature you control. That creature gets +0/+X until end of turn, where X is its mana value.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    granted_preventions: &[prevention::DAMAGE_DEALT_BY.text("Prevent all damage that would be dealt this turn by target creature you control. That creature gets +0/+X until end of turn, where X is its mana value.")],
    ..oracle::KRY_SHIELD
};
