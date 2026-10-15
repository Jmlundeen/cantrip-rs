use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect: effects::draw::<2> }),
    // As an additional cost to cast this spell, sacrifice a creature (CR 118.8).
    additional_cost: Some(Cost::sacrifice(1, targets::filters::creature)),
    ..oracle::VILLAGE_RITES
};
