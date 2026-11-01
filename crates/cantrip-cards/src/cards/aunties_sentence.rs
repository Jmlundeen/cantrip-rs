use cantrip_api::*;
use cantrip_mechanics::hand::target_opponent_discards_chosen_nonland_permanent;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

static MODAL: ModalDef = modes(&[
    mode(target_opponent_discards_chosen_nonland_permanent, "Target opponent reveals their hand. You choose a nonland permanent card from it. That player discards that card.")
        .targets(&[targets::OPPONENT]),
    mode(effects::target_creature_gets::<-2, -2>, "Target creature gets -2/-2 until end of turn.").targets(&[targets::CREATURE]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::AUNTIES_SENTENCE };
