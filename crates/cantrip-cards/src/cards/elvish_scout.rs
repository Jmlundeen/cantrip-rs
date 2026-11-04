use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{G}").and(TAP), prevention::untap_target_and_prevent_combat_damage_to_and_by_it,
        "{G}, {T}: Untap target attacking creature you control. Prevent all combat damage that would be dealt to and dealt by it this turn.")
        .targets(&[targets::ATTACKING_CREATURE_YOU_CONTROL])],
    granted_preventions: &[prevention::COMBAT_DAMAGE_DEALT_TO_AND_BY
        .text("Prevent all combat damage that would be dealt to and dealt by it this turn.")],
    ..oracle::ELVISH_SCOUT
};
