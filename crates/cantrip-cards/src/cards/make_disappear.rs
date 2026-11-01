use cantrip_api::*;
use cantrip_mechanics::{casualty, effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effects::counter_first_target_unless_pays::<2>),
    optional_cost: Some(casualty::optional::<1>()),
    labels: &["Cast without casualty", "Casualty 1: sacrifice a creature with power 1 or greater"],
    abilities: &[casualty::trigger("Casualty 1 (As you cast this spell, you may sacrifice a creature with power 1 or greater. When you do, copy this spell and you may choose a new target for the copy.)")],
    ..oracle::MAKE_DISAPPEAR
};
