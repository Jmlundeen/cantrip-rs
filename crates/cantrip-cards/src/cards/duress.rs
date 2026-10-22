use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{choose, select, targets};

/// "A noncreature, nonland card": the quality of the card Duress takes, each card's own.
fn noncreature_nonland(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |card| {
        let chars = q.chars(card);
        !chars.is(CardType::Creature) && !chars.is(CardType::Land)
    })
}

/// "You choose a noncreature, nonland card from it": one, if the hand has one (CR 608.2d), with the whole hand shown.
fn question(item: &StackItem, hand: Vec<ObjectId>) -> RequestKind {
    let rule = Selection::new(select::as_many_as_can(1)).with(select::part(&(noncreature_nonland as SelectFn), 0, ""));
    choose::select_question(item.object, Purpose::Discard, hand, rule)
}

/// Target opponent reveals their hand; you choose a noncreature, nonland card from it; they discard it. Asked even
/// when nothing in the hand is eligible: looking at the hand is part of the spell.
#[cfg(not(feature = "staged"))]
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    if hand.is_empty() {
        return Ok(());
    }
    g.reveal(opponent, &hand);
    let ask = question(item, hand);
    if let Answer::Cards(cards) = g.ask(item.controller, ask)? {
        g.discard_together(&cards);
    }
    Ok(())
}

/// The same, as explicit stages: stage 0 reveals and asks, stage 1 discards what was chosen.
#[cfg(feature = "staged")]
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.target(0).player() else { return Ok(()) };
    match g.stage() {
        0 => {
            let hand = g.state().zone(Zone::hand(opponent)).to_vec();
            if hand.is_empty() {
                return Ok(());
            }
            g.reveal(opponent, &hand);
            let ask = question(item, hand);
            Err(g.ask_then(1, item.controller, ask))
        }
        _ => {
            if let Some(Answer::Cards(cards)) = g.staged_answer() {
                g.discard_together(&cards);
            }
            Ok(())
        }
    }
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::DURESS
};
