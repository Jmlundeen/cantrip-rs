use cantrip_api::*;
use cantrip_mechanics::discard_chosen;
use cantrip_oracle as oracle;

const WATERBEND: Question =
    Question::new("Waterbend", "waterbend {cost}, or discard a card?", "Waterbending Lesson: then discard a card unless you waterbend {2}", "Waterbend {cost}", "Discard a card").no_default();

/// "Draw three cards. Then discard a card unless you waterbend {2}." The cards are drawn first (CR 608.2c, "then"), so the one
/// discarded may be one of them. Paying is a choice (CR 701.67a, 701.67b: {2} of tapped artifacts and creatures, or mana); the
/// discard is what is left when it is declined or cannot be paid.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    g.draw(player, 3);
    g.end_sentence();
    if !g.may_pay(player, item.object, item.object, Cost::waterbend(2, 0), &WATERBEND)? {
        discard_chosen(g, player, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::WATERBENDING_LESSON };
