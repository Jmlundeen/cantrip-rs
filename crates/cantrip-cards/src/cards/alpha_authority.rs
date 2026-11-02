use cantrip_api::*;
use cantrip_mechanics::{attach, combat::one_blocker_at_most, statics::have_hexproof, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Ability,
        attach::attached_object,
        have_hexproof,
        "Enchanted creature has hexproof and can't be blocked by more than one creature.",
    )],
    enchant: Some(targets::CREATURE),
    blocker_limits: &[one_blocker_at_most],
    ..oracle::ALPHA_AUTHORITY
};
