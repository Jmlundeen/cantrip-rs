use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{discard_chosen, sacrifice_chosen, targets, triggers};
use cantrip_oracle as oracle;

fn creature_or_planeswalker(chars: &Chars) -> bool {
    chars.is(CardType::Creature) || chars.is(CardType::Planeswalker)
}

/// Target opponent sacrifices a creature or planeswalker of their choice, discards a card, and loses 3 life. You draw a
/// card and gain 3 life.
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(opponent) = item.targets[0] else { return Ok(()) };
    sacrifice_chosen(g, opponent, item.object, 1, creature_or_planeswalker)?;
    discard_chosen(g, opponent, item.object, 1)?;
    g.lose_life(opponent, 3);
    g.draw(item.controller, 1);
    g.gain_life(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, drain, "When Archon of Cruelty enters, target opponent sacrifices a creature or planeswalker of their choice, discards a card, and loses 3 life. You draw a card and gain 3 life.")
            .targets(&[targets::OPPONENT]),
        triggered(triggers::THIS_ATTACKS, drain, "Whenever Archon of Cruelty attacks, target opponent sacrifices a creature or planeswalker of their choice, discards a card, and loses 3 life. You draw a card and gain 3 life.")
            .targets(&[targets::OPPONENT]),
    ],
    ..oracle::ARCHON_OF_CRUELTY
};
