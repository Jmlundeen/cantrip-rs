use cantrip_api::*;
use cantrip_mechanics::effects::destroy_first_target;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // Destroy target creature (CR 701.8); nothing if it is no longer a legal target (CR 608.2b), the sacrifice paid all the
    // same.
    spell: spell(&[targets::CREATURE], destroy_first_target),
    // As an additional cost to cast this spell, sacrifice a creature (CR 118.8, 601.2h): exactly one.
    additional_cost: Some(Cost::sacrifice(1, targets::filters::creature)),
    ..oracle::BONE_SPLINTERS
};
