use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, flurry, targets};
use cantrip_oracle as oracle;

/// "Target nonland permanent card with mana value 2 or less from your graveyard" (CR 115.1, 110.4a, 202.3): a land card is
/// not one, and X in a graveyard is 0 (CR 202.3e).
fn nonland_permanent_card_with_mana_value_2_or_less_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::nonland_permanent_card_in_your_graveyard(q, asker, t) && q.mana_value(id) <= 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            effects::return_first_target_to_hand,
            "When this creature enters, return target nonland permanent card with mana value 2 or less from your graveyard to your hand.",
        )
        .targets(&[target(nonland_permanent_card_with_mana_value_2_or_less_in_your_graveyard)]),
        flurry::trigger(
            "Flurry \u{2014} Whenever you cast your second spell each turn, tap target creature an opponent controls.",
            effects::tap_targets,
        )
        .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::WAYSPEAKER_BODYGUARD
};
