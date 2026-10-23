use cantrip_api::*;
use cantrip_mechanics::{damage_target, effects, targets};
use cantrip_oracle as oracle;

/// "If this spell was kicked, draw a card. Fires of Victory deals damage to target creature or planeswalker equal to the
/// number of cards in your hand." (CR 702.33e, 608.2c, 608.2h): the draw comes first, so the card drawn counts; the spell itself
/// is on the stack, not in the hand.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if item.kicked > 0 {
        effects::draw(g, item, 1)?;
        g.end_sentence();
    }
    let cards = u16::try_from(g.state().zone(Zone::hand(item.controller)).len()).unwrap_or(u16::MAX);
    damage_target(g, item, 0, cards);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: kicker(mana("{2}{U}")),
    spell: spell(&[targets::CREATURE_OR_PLANESWALKER], effect),
    ..oracle::FIRES_OF_VICTORY
};
