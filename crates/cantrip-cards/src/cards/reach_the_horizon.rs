use cantrip_api::*;
use cantrip_mechanics::library;
use cantrip_mechanics::select::{self, Selection};
use cantrip_oracle as oracle;

/// "Up to two basic land cards and/or Town cards with different names" (CR 201.2b, 701.23b).
const BASIC_LANDS_AND_TOWNS_WITH_DIFFERENT_NAMES: Selection =
    Selection::of(select::up_to(2), &[select::basic_land_or_subtype(Subtype::Town), select::DIFFERENT_NAMES]).called("basic lands and/or Towns with different names");

/// "Search your library for up to two basic land cards and/or Town cards with different names, put them onto the battlefield
/// tapped, then shuffle" (CR 701.23, 701.24): fewer may be found, or none, and the library is shuffled either way.
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, BASIC_LANDS_AND_TOWNS_WITH_DIFFERENT_NAMES, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], search), ..oracle::REACH_THE_HORIZON };
