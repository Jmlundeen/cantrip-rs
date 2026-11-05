use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS_OR_ATTACKS,
        effects::may_return_first_target_to_battlefield,
        "Whenever this creature enters or attacks, you may return target permanent card with mana value 3 or less from your graveyard to the battlefield.",
    )
    .targets(&[targets::PERMANENT_CARD_IN_YOUR_GRAVEYARD_WITH_MANA_VALUE_3_OR_LESS])],
    ..oracle::SUN_TITAN
};
