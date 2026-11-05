use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one or both" (CR 700.2): the modes are carried out in the order printed (CR 608.2c).
static MODAL: ModalDef = modes(&[
    mode(effects::damage_first_target::<3>, "Overwhelming Surge deals 3 damage to target creature.").targets(&[targets::CREATURE]),
    mode(effects::destroy_first_target, "Destroy target noncreature artifact.").targets(&[targets::NONCREATURE_ARTIFACT]),
])
.choose(1, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::OVERWHELMING_SURGE };
