use cantrip_api::*;
use cantrip_mechanics::{prevention, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ATTACKING_CREATURE], prevention::all_combat_damage_dealt_by_target_this_turn),
    granted_preventions: &[GrantedPreventionDef {
        text: "Prevent all combat damage that would be dealt by target attacking creature this turn.",
        ..prevention::COMBAT_DAMAGE_DEALT_BY
    }],
    ..oracle::WARNING
};
