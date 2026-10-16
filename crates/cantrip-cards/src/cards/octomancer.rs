use cantrip_api::*;
use cantrip_mechanics::{gift, history, targets, tokens, triggers::EACH_END_STEP};
use cantrip_oracle as oracle;

/// "Target creature token that entered the battlefield this turn" (CR 115.1): a creature token on the battlefield that
/// entered it this turn, under any player's control.
fn creature_token_that_entered_this_turn(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.obj(id).token && history::entered_this_turn(q, id))
}

/// "Gift an Octopus" (CR 702.174i): the 8/8 blue Octopus is a token of its own, and its gift triggered ability (CR 702.174b)
/// gives it as Octomancer enters.
pub static EXTRA: &[&CardDef] = &[&tokens::OCTOPUS];

pub static CARD: CardDef = CardDef {
    gift: Some(gift::OCTOPUS),
    abilities: &[
        gift::trigger("When this creature enters, if its gift cost was paid, the chosen opponent creates an 8/8 blue Octopus creature token."),
        triggered(EACH_END_STEP, cantrip_mechanics::copy::create_token_copy_of_target, "At the beginning of each end step, create a token that's a copy of target creature token that entered the battlefield this turn.")
            .targets(&[target(creature_token_that_entered_this_turn)]),
    ],
    ..oracle::OCTOMANCER
};
