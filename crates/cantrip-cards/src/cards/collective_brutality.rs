use cantrip_api::*;
use cantrip_mechanics::escalate;
use cantrip_mechanics::{pump, targets, hand};
use cantrip_oracle as oracle;

/// Target opponent reveals their hand. You choose an instant or sorcery card from it. That player discards that card.
/// Asked even when nothing in the hand is eligible: looking at the hand is part of the spell.
fn strip(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    hand::discard_chosen_from_revealed_hand(g, item, opponent, Purpose::Discard, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Instant) || chars.is(CardType::Sorcery)
    })?;
    Ok(())
}

/// Target creature gets -2/-2 until end of turn.
fn shrink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, -2, -2);
    }
    Ok(())
}

/// Target opponent loses 2 life and you gain 2 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(opponent) = item.targets[0].player() {
        g.lose_life(opponent, 2);
    }
    g.gain_life(item.controller, 2);
    Ok(())
}

/// Escalate (CR 702.120): discard a card for each mode chosen beyond the first.
const MODES: &[ModeDef] = &[
    mode(strip, "Target opponent reveals their hand. You choose an instant or sorcery card from it. That player discards that card.")
        .targets(&[targets::OPPONENT]),
    mode(shrink, "Target creature gets -2/-2 until end of turn.")
        .targets(&[targets::CREATURE]),
    mode(drain, "Target opponent loses 2 life and you gain 2 life.")
        .targets(&[targets::OPPONENT]),
];

static MODAL: ModalDef = escalate::modal(MODES, Cost { discard: Some((1, targets::filters::any)), ..Cost::NONE });

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::COLLECTIVE_BRUTALITY };
