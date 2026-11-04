use cantrip_api::*;
use cantrip_mechanics::effects::{plus_one_counter_on_this, return_first_target_to_hand};
use cantrip_mechanics::history;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::{THIS_ENTERS, YOUR_END_STEP};
use cantrip_oracle as oracle;

/// "Target artifact or creature card from your graveyard" (CR 115.1): a card of either type in its controller's own graveyard.
fn artifact_or_creature_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Artifact, CardType::Creature])
}

/// Flying is in the content JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            return_first_target_to_hand,
            "When this creature enters, return target artifact or creature card from your graveyard to your hand.",
        )
        .targets(&[target(artifact_or_creature_card_in_your_graveyard)]),
        // "Void" (CR 207.2c, 603.4): an intervening "if", checked as the end step begins and again as the ability resolves.
        triggered(
            YOUR_END_STEP,
            plus_one_counter_on_this,
            "Void \u{2014} At the beginning of your end step, if a nonland permanent left the battlefield this turn or a spell was warped this turn, put a +1/+1 counter on this creature.",
        )
        .condition(|q, _| history::void(q)),
    ],
    ..oracle::INTERCEPTOR_MECHAN
};
