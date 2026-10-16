use cantrip_api::*;
use cantrip_mechanics::{choose, haste_until_end_of_turn, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "You may discard a card. If you do, draw two cards." Choosing no card declines; the draw follows a card that was
/// actually discarded. Nothing is asked of an empty hand.
fn discard_to_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hand = g.state().zone(Zone::hand(item.controller)).to_vec();
    if let Some(&card) = choose::cards(g, item.controller, item.object, Purpose::Discard, hand, 1, true)?.first() {
        g.discard(card);
        g.draw(item.controller, 2);
    }
    Ok(())
}

/// Creatures you control gain haste until end of turn.
fn haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for id in creatures {
        haste_until_end_of_turn(g, item.object, id);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, discard_to_draw, "When Bitter Reunion enters, you may discard a card. If you do, draw two cards."),
        ability(cost("{1}").and(SACRIFICE_THIS), haste, "{1}, Sacrifice Bitter Reunion: Creatures you control gain haste until end of turn."),
    ],
    ..oracle::BITTER_REUNION
};
