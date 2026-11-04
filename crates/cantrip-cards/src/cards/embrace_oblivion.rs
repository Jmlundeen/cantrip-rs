use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // Destroy target creature or Spacecraft (CR 701.8): a Spacecraft that is no creature is a legal target too.
    spell: spell(&[targets::CREATURE_OR_SPACECRAFT], effects::destroy_first_target),
    // As an additional cost to cast this spell, sacrifice an artifact or creature (CR 118.8, 601.2b).
    additional_cost: Some(Cost::sacrifice(1, targets::filters::artifact_or_creature)),
    ..oracle::EMBRACE_OBLIVION
};
