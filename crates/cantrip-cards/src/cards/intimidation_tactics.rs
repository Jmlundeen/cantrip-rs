use cantrip_api::*;
use cantrip_mechanics::{hand::choose_from_revealed_hand_of_kind, targets, triggers::cycling};
use cantrip_oracle as oracle;

/// "Target opponent reveals their hand. You choose an artifact or creature card from it. Exile that card." The reveal is
/// the looking, so nothing is asked when no card in the hand is an artifact or a creature. The card is exiled, not
/// discarded (CR 701.13).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    let chosen = choose_from_revealed_hand_of_kind(g, item, opponent, Purpose::Exile, "artifact or creature", |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Artifact) || chars.is(CardType::Creature)
    })?;
    g.exile_all(&chosen);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::OPPONENT], effect }),
    abilities: &[cycling(mana(3, 0, 0, 0, 0, 0), "Cycling {3} ({3}, Discard this card: Draw a card.)")],
    ..oracle::INTIMIDATION_TACTICS
};
