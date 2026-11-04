use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When this creature enters, choose one" (CR 700.2, 603.3c). Puff Piece: a +1/+1 counter on each of up to two target creatures
/// (CR 122.1, 601.2c), which may be none at all. Investigative Journalism: return target creature card with mana value 2 or less
/// from your graveyard to your hand (CR 400.3). Each mode has the targets of its own (CR 700.2c).
static MODAL: ModalDef = modes(&[
    mode(effects::plus_one_counter_on_each_target, "Puff Piece \u{2014} Put a +1/+1 counter on each of up to two target creatures.")
        .targets(&[targets::CREATURE.between(0, 2)]),
    mode(
        effects::return_first_target_to_hand,
        "Investigative Journalism \u{2014} Return target creature card with mana value 2 or less from your graveyard to your hand.",
    )
    .targets(&[targets::CREATURE_CARD_WITH_MANA_VALUE_2_OR_LESS_IN_YOUR_GRAVEYARD]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When this creature enters, choose one \u{2014}\n\u{2022} Puff Piece \u{2014} Put a +1/+1 counter on each of up to two target creatures.\n\u{2022} Investigative Journalism \u{2014} Return target creature card with mana value 2 or less from your graveyard to your hand.",
    )],
    ..oracle::DAILY_BUGLE_REPORTERS
};
