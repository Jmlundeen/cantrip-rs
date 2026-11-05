use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one" (CR 700.2): each mode has its own target (CR 115.1), "creature with flying" being checked as the layers give
/// it now.
static MODAL: ModalDef = modes(&[
    mode(effects::target_creature_gets::<3, 3>, "Target creature gets +3/+3 until end of turn.").targets(&[targets::CREATURE]),
    mode(effects::destroy_first_target, "Destroy target creature with flying.").targets(&[targets::CREATURE_WITH_FLYING]),
])
.choose(1, 1);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SARKHANS_RESOLVE };
