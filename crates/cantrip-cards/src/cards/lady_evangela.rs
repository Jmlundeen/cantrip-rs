use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{W}{B}").and(TAP), prevention::all_combat_damage_dealt_by_target_this_turn,
        "{W}{B}, {T}: Prevent all combat damage that would be dealt by target creature this turn.")
        .targets(&[targets::CREATURE])],
    granted_preventions: &[GrantedPreventionDef {
        text: "Prevent all combat damage that would be dealt by target creature this turn.",
        ..prevention::COMBAT_DAMAGE_DEALT_BY
    }],
    ..oracle::LADY_EVANGELA
};
