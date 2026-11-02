use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, untap};
use cantrip_oracle as oracle;

/// "Draw three cards, untap up to two lands, then discard a card." (CR 608.2, 121.1, 701.9a).
fn draw_three_cards_untap_up(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 3);
    g.end_sentence();
    untap::up_to_lands(g, item.object, item.controller, 2)?;
    g.end_sentence();
    discard_chosen(g, item.controller, item.object, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], draw_three_cards_untap_up),
    ..oracle::PORE_OVER_THE_PAGES
};
