use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, prevention::untap_target_and_prevent_combat_damage_to_and_by_it, "{T}: Untap target attacking creature. Prevent all combat damage that would be dealt to and dealt by that creature this turn.")
        .targets(&[targets::ATTACKING_CREATURE])],
    granted_preventions: &[prevention::COMBAT_DAMAGE_DEALT_TO_AND_BY],
    ..oracle::MAZE_OF_ITH
};
