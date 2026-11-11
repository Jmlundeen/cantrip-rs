use cantrip_api::*;
use cantrip_mechanics::delayed;
use cantrip_oracle as oracle;

/// "Another nontoken creature" (CR 109.1, 111.1, 701.21a): a creature the paying player controls other than this artifact
/// creature itself, and not a token - it is what the ability's cost sacrifices, and a token would cease to exist rather than
/// be a card to return.
fn another_nontoken_creature(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    id != asker.source && q.chars(id).is(CardType::Creature) && !q.state().obj(id).token
}

/// "{4}, Sacrifice another nontoken creature: Return that card to the battlefield under its owner's control at the beginning
/// of the next end step." (CR 118.3, 608.2h, 603.7c, 400.7): the sacrifice is part of the cost, paid as the ability is
/// activated, and the delayed ability is about the card the sacrificed permanent was - carried by object identity, so a card
/// that left the graveyard in between is a new object and does not return.
fn return_sacrificed_at_next_end_step(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(&sacrificed) = g.cost_objects(item).first() else { return Ok(()) };
    // "That card" is the new object the sacrificed permanent became in the graveyard (CR 400.7): the delayed ability is set
    // up about it, and returns it under its owner's control (CR 110.2a). A token has ceased to exist (CR 111.7), and the
    // ability finds nothing.
    let Some(card) = g.found_after_move(sacrificed) else { return Ok(()) };
    delayed::at_next_end_step(g, item, 0, Target::Object(card));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{4}").and(sacrifice(1, another_nontoken_creature)),
        return_sacrificed_at_next_end_step,
        "{4}, Sacrifice another nontoken creature: Return that card to the battlefield under its owner's control at the beginning of the next end step.",
    )],
    delayed: &[delayed_trigger(
        delayed::return_subject_to_battlefield,
        "Return that card to the battlefield under its owner's control at the beginning of the next end step.",
    )],
    ..oracle::THE_GUARDIAN_OF_FOREVER
};
