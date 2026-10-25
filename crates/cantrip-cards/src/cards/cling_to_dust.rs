use cantrip_api::*;
use cantrip_mechanics::{effects, escape, targets};
use cantrip_oracle as oracle;

/// "Exile target card from a graveyard. If it was a creature card, you gain 3 life. Otherwise, you draw a card." The card's
/// types are read before it leaves (CR 400.7, 608.2h); a card that is a creature and another type is a creature card, so
/// it gains life and does not draw. With the target illegal on resolution the core does not resolve the spell (CR 608.2b).
fn exile_and_gain_or_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let creature = g.chars(card).is(CardType::Creature);
    g.exile(card);
    g.end_sentence();
    if creature {
        effects::gain_life(g, item, 3)?;
    } else {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CARD_IN_A_GRAVEYARD], exile_and_gain_or_draw),
    ways: &[escape::way(mana("{3}{B}"), 5, "Escape—{3}{B}, Exile five other cards from your graveyard")],
    ..oracle::CLING_TO_DUST
};
