use cantrip_api::*;
use cantrip_mechanics::tokens::WHITE_FLYING_SPIRIT;
use cantrip_mechanics::choose_from_hand;
use cantrip_oracle as oracle;

/// Draw X cards, then discard X cards (mandatory, one batch). A Spirit for each card type among the cards discarded
/// this way, created together (CR 101.4). Types are read before the discard, since a card is a new object after it
/// moves (CR 400.7).
fn epiphany(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = item.x;
    if x < 1 {
        return Ok(());
    }
    let you = item.controller;
    g.draw(you, x as u32);
    let chosen = choose_from_hand(g, you, item.object, x.min(255) as u8)?;
    let mut types = 0;
    for kind in CardType::ALL {
        if chosen.iter().any(|&c| g.chars(c).is(kind)) {
            types += 1;
        }
    }
    g.discard_together(&chosen);
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
