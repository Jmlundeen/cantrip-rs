use cantrip_api::*;
use cantrip_mechanics::hand::target_opponent_discards_chosen_nonland_permanent;
use cantrip_mechanics::{earthbend, targets};
use cantrip_oracle as oracle;

static MODAL: ModalDef = modes(&[
    mode(target_opponent_discards_chosen_nonland_permanent, "Target opponent reveals their hand. You choose a nonland permanent card from it. That player discards that card.")
        .targets(&[targets::OPPONENT]),
    mode(earthbend::effect::<2>, "Earthbend 2.").targets(&[earthbend::TARGET]),
]);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&MODAL),
    delayed: &[earthbend::RETURN],
    ..oracle::DAI_LI_INDOCTRINATION
};
