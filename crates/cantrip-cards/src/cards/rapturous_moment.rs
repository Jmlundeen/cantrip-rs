use cantrip_api::*;
use cantrip_mechanics::discard_chosen;
use cantrip_oracle as oracle;

/// "Draw three cards, then discard two cards. Add {U}{U}{R}{R}{R}." (CR 121.1, 701.9, 106.4): the draw, the discard of two cards
/// the controller chooses (all of them if they hold fewer), then the mana, in the order the text gives; the mana empties from the
/// pool as the phase ends (CR 500.5).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 3);
    discard_chosen(g, item.controller, item.object, 2)?;
    g.add_mana(item.controller, Mana::U, 2);
    g.add_mana(item.controller, Mana::R, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::RAPTUROUS_MOMENT };
