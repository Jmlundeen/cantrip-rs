use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{2}").and(TAP),
            prevention::untap_target_and_prevent_combat_damage_to_and_by_it,
            "{2}, {T}: Untap target attacking creature you control. Prevent all combat damage that would be dealt to and dealt by that creature this turn.",
        )
        .targets(&[targets::ATTACKING_CREATURE_YOU_CONTROL]),
    ],
    granted_preventions: &[prevention::COMBAT_DAMAGE_DEALT_TO_AND_BY],
    ..oracle::EBONY_HORSE
};
