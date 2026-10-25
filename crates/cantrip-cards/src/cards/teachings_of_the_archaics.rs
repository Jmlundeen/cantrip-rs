use cantrip_api::*;
use cantrip_mechanics::{cards_in_hand, effects};
use cantrip_oracle as oracle;

/// "If an opponent has more cards in hand than you, draw two cards. Draw three cards instead if an opponent has at least
/// four more cards in hand than you." The hands are counted as the spell resolves (it is on the stack, not in a hand), and
/// each opponent is compared with you on their own. Three replaces the two: the cards are drawn once.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let mine = cards_in_hand(g, you) as i64;
    let most = g.state().opponents(you).map(|opponent| cards_in_hand(g, opponent) as i64).max();
    match most {
        Some(theirs) if theirs >= mine + 4 => effects::draw(g, item, 3)?,
        Some(theirs) if theirs > mine => effects::draw(g, item, 2)?,
        _ => {}
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::TEACHINGS_OF_THE_ARCHAICS };
