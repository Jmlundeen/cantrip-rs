use cantrip_api::*;
use cantrip_mechanics::{effects, library, select, targets};
use cantrip_oracle as oracle;

/// "Search your library for an artifact card, reveal it, put it into your hand, then shuffle" (CR 701.23). The search may come
/// up empty; the shuffle follows either way.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, select::ARTIFACT_CARD, true)?;
    Ok(())
}

/// "Choose one or both" (CR 700.2): the modes are done in printed order (CR 700.2d); the second has its own target
/// (CR 700.2c), and one whose target is gone by resolution is left alone, the search still being done (CR 608.2b).
static MODAL: ModalDef = modes(&[
    mode(search, "Search your library for an artifact card, reveal it, put it into your hand, then shuffle."),
    mode(effects::return_first_target_to_hand, "Return target artifact card from your graveyard to your hand.")
        .targets(&[targets::ARTIFACT_CARD_IN_YOUR_GRAVEYARD]),
])
.choose(1, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SCOUR_FOR_SCRAP };
