use cantrip_api::*;
use cantrip_mechanics::library::{search_to_graveyard, Find};
use cantrip_oracle as oracle;

/// Search your library for a card, put that card into your graveyard, then shuffle.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_graveyard(g, item.controller, item.object, Find::ANY, |_, _| true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::ENTOMB };
