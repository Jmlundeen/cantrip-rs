use cantrip_api::*;
use cantrip_mechanics::escalate;
use cantrip_mechanics::{effects, targets, hand};
use cantrip_oracle as oracle;

/// Target opponent reveals their hand. You choose an instant or sorcery card from it. That player discards that card.
/// Asked even when nothing in the hand is eligible: looking at the hand is part of the spell.
fn strip(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    hand::discard_chosen_from_revealed_hand(g, item, opponent, Purpose::Discard, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Instant) || chars.is(CardType::Sorcery)
    })?;
    Ok(())
}

/// Escalate (CR 702.120): discard a card for each mode chosen beyond the first.
const MODES: &[ModeDef] = &[
    mode(strip, "Target opponent reveals their hand. You choose an instant or sorcery card from it. That player discards that card.")
        .targets(&[targets::OPPONENT]),
    mode(effects::target_creature_gets::<-2, -2>, "Target creature gets -2/-2 until end of turn.")
        .targets(&[targets::CREATURE]),
    mode(|g, item| effects::target_player_loses_life_you_gain_life(g, item, 2), "Target opponent loses 2 life and you gain 2 life.")
        .targets(&[targets::OPPONENT]),
];

static MODAL: ModalDef = escalate::modal(MODES, discard_cards(1));

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::COLLECTIVE_BRUTALITY };
