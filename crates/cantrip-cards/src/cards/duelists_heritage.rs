use cantrip_api::*;
use cantrip_mechanics::effects::{ask_may, gain_until_end_of_turn};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Whenever one or more creatures attack" (CR 508.1m, 603.2): one declaration, whatever the number of attackers and
/// whichever player declares them, is one trigger.
fn creatures_attack(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    matches!(event, Event::AttackersDeclared { .. }).then(|| Fired::PLAIN).flatten()
}

const ASK: Question = Question::new("GainDoubleStrike", "have the attacking creature gain double strike until end of turn?", "Chosen as the ability resolves", "Gain double strike", "Don't");

/// "you may have target attacking creature gain double strike until end of turn" (CR 702.4, 611.2a, 608.2b): the target is
/// mandatory and chosen as the trigger goes on the stack; the "may" is asked as it resolves, and only if the creature is
/// still an attacking creature (a legal target).
fn grant_double_strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    if ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? {
        gain_until_end_of_turn(g, item.object, creature, Keyword::DoubleStrike);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::ATTACKERS_DECLARED, creatures_attack),
        grant_double_strike,
        "Whenever one or more creatures attack, you may have target attacking creature gain double strike until end of turn.",
    )
    .targets(&[targets::ATTACKING_CREATURE])],
    ..oracle::DUELISTS_HERITAGE
};
