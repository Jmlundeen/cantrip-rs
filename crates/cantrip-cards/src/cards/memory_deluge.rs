use cantrip_api::*;
use cantrip_mechanics::{flashback, library};
use cantrip_oracle as oracle;

/// "Look at the top X cards of your library, where X is the amount of mana spent to cast this spell. Put two of them into your
/// hand and the rest on the bottom of your library in a random order." (CR 601.2h, 608.2).
fn look_top_x_cards_library(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let spent = g.obj(item.object).spent.total();
    library::look_at_top(g, item.controller, item.object, spent as usize, (2, 2), false, |_| true, library::Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], look_top_x_cards_library),
    ways: &[flashback::way(cost("{5}{U}{U}"), "Flashback {5}{U}{U}")],
    ..oracle::MEMORY_DELUGE
};
