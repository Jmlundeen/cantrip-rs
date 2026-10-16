use cantrip_api::*;
use cantrip_oracle as oracle;

/// "During turns other than yours, this spell costs {3} more to cast." (CR 601.2f): `player` is the one casting it.
fn three_more_on_other_turns(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed) -> i8 {
    if matches!(costed, Costed::Spell(_)) && q.state().turn.active != player { 3 } else { 0 }
}

/// "Return all nonland permanents to their owners' hands. End the turn." Every nonland permanent goes together (a token
/// ceases to exist in a hand, CR 704.5d), then the turn ends (CR 724.1): the leaves-the-battlefield abilities that
/// triggered cease to exist (724.1a), and the spell is exiled from the stack with every other spell and ability (724.1b).
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let nonland: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| !g.chars(id).is(CardType::Land)).collect();
    g.return_to_hand(&nonland);
    g.end_the_turn()
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    own_cost_changes: &[three_more_on_other_turns],
    ..oracle::HURKYLS_FINAL_MEDITATION
};
