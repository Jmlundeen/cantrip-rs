use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

static MODAL: ModalDef = modes(&[
    mode(effects::target_creature_gets::<3, 2>, "Use the Relic \u{2014} Target creature gets +3/+2 until end of turn.")
        .targets(&[targets::CREATURE]),
    mode(effects::destroy_first_target, "Destroy the Relic \u{2014} Destroy target artifact.").targets(&[targets::ARTIFACT]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::PERILS_OF_THE_PAST };
