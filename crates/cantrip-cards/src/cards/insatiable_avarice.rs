use cantrip_api::*;
use cantrip_mechanics::library::{Find, search_to_top};
use cantrip_mechanics::{spree, targets};
use cantrip_oracle as oracle;

/// Mode 0: "Search your library for a card, then shuffle and put that card on top." (CR 701.23, 701.24) Any card, so
/// one is always found if the library has any.
fn tutor(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_top(g, item.controller, item.object, Find::ANY, |_, _| true)?;
    Ok(())
}

/// Mode 1: "Target player draws three cards and loses 3 life." (CR 121.1, 119.3) With its target illegal it does nothing.
fn draw_and_lose(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        g.draw(player, 3);
        g.lose_life(player, 3);
    }
    Ok(())
}

const MODES: &[ModeDef] = &[
    mode(tutor, "+ {2} \u{2014} Search your library for a card, then shuffle and put that card on top.").cost(mana("{2}")),
    mode(draw_and_lose, "+ {B}{B} \u{2014} Target player draws three cards and loses 3 life.").targets(&[targets::PLAYER]).cost(mana("{B}{B}")),
];

static MODAL: ModalDef = spree::modal(MODES);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::INSATIABLE_AVARICE };
