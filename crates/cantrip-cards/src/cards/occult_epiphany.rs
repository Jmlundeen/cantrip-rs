use cantrip_api::*;
use cantrip_mechanics::tokens::WHITE_FLYING_SPIRIT;
use cantrip_mechanics::{choose_from_hand, hand};
use cantrip_oracle as oracle;

/// Draw X cards, then discard X cards (mandatory, one batch). A Spirit for each card type among the cards discarded
/// this way, created together (CR 101.4). Each card's types are read by its discard where it went
/// (`hand::discarded_chars`): one put into a hidden zone unrevealed is undefined and adds no type (CR 701.9c).
fn epiphany(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = item.x;
    if x < 1 {
        return Ok(());
    }
    let you = item.controller;
    g.draw(you, x as u32);
    let chosen = choose_from_hand(g, you, item.object, u32::from(x))?;
    let discarded = g.discard_together(&chosen);
    let types = hand::card_types_among_discarded(&*g, &discarded);
    if types == 0 {
        return Ok(());
    }
    let spirit = g.card_id(&WHITE_FLYING_SPIRIT);
    let tokens: Vec<(CardId, PlayerId)> = (0..types).map(|_| (spirit, you)).collect();
    g.create_tokens_together(&tokens)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], epiphany),
    ..oracle::OCCULT_EPIPHANY
};

pub static EXTRA: &[&CardDef] = &[&WHITE_FLYING_SPIRIT];
