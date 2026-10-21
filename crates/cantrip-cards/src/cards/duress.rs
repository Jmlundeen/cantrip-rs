use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::targets;

fn eligible(g: &dyn Query, hand: &[ObjectId]) -> Vec<ObjectId> {
    hand.iter()
        .copied()
        .filter(|&c| {
            let chars = g.chars(c);
            !chars.is(CardType::Creature) && !chars.is(CardType::Land)
        })
        .collect()
}

fn question(g: &dyn Query, item: &StackItem, hand: Vec<ObjectId>) -> RequestKind {
    let eligible = eligible(g, &hand);
    let count = eligible.len().min(1) as u8;
    RequestKind::ChooseCards { source: item.object, purpose: Purpose::Discard, cards: hand, eligible, count, up_to: false, kind: "", total: None }
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
    let ask = question(g, item, hand);
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
            let ask = question(g, item, hand);
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
