use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::CREATURE],
        prevention::prevent_damage_by_target_then_toughness_equal_to_mana_value,
    ),
    granted_preventions: &[prevention::COMBAT_DAMAGE_DEALT_BY.text("Prevent all combat damage that would be dealt by target creature this turn. That creature gets +0/+X until end of turn, where X is its mana value.")],
    ..oracle::SUBDUE
};
