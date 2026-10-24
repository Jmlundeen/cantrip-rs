use cantrip_api::*;
use cantrip_mechanics::{choose, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Up to one target creature they control": controlled by the very opponent the first slot chose (CR 115.1, 601.2c),
/// not merely by some opponent.
fn creature_they_control(q: &dyn Query, _: &Asker, t: Target) -> bool {
    let Target::Object(id) = t else { return false };
    let Some(Target::Player(opponent)) = q.chosen_targets().first().copied() else { return false };
    targets::permanent(q, id) && q.chars(id).is(CardType::Creature) && q.controller(id) == opponent
}

/// They reveal their hand (CR 701.20). "You may exile a nonland card from their hand or the chosen creature" is one
/// choice over both, since the card takes one of them or nothing; it is exiled until this leaves the battlefield
/// (CR 610.3) and returns the moment it has.
fn reveal_and_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    if !hand.is_empty() {
        g.reveal(opponent, &hand);
    }
    let mut shown = hand.clone();
    let mut eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| !g.chars(c).is(CardType::Land)).collect();
    if let Some(creature) = targets::object(g, item, 1) {
        shown.push(creature);
        eligible.push(creature);
    }
    let cards = choose::cards_from(g, item.controller, item.object, Purpose::Exile, shown, eligible, 1, true)?;
    g.exile_with(&cards, g.linked(item), true);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, reveal_and_exile, "When Cloak and Dagger enter, choose target opponent and up to one target creature they control. They reveal their hand. You may exile a nonland card from their hand or the chosen creature until Cloak and Dagger leave the battlefield.")
        .targets(&[targets::OPPONENT, up_to_one(creature_they_control)])],
    ..oracle::CLOAK_AND_DAGGER_ENTWINED
};
