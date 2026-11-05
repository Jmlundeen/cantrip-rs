use cantrip_api::*;
use cantrip_mechanics::effects::gain_until_end_of_turn;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Whenever one or more creatures attack" (CR 508.1m, 603.2): one declaration, whatever the number of attackers and
/// whichever player declares them, is one trigger.
fn creatures_attack(_: &dyn Query, _: ObjectId, event: &Event) -> Option<Fired> {
    matches!(event, Event::AttackersDeclared { .. }).then(|| Fired::PLAIN).flatten()
}

/// "you may have target attacking creature gain double strike until end of turn" (CR 702.4, 611.2a): the target is
/// chosen as the trigger goes on the stack, and the choice is made as it resolves; a creature no longer attacking is
/// an illegal target and the ability does nothing (CR 608.2b).
fn grant_double_strike(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
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
    .targets(&[targets::ATTACKING_CREATURE.optional()])],
    ..oracle::DUELISTS_HERITAGE
};
