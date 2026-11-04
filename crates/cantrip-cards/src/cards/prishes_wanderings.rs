use cantrip_api::*;
use cantrip_mechanics::select::{self, Selection};
use cantrip_mechanics::{effects, library, targets};
use cantrip_oracle as oracle;

/// "A basic land card or Town card", which may be left unfound (CR 701.23b).
const BASIC_LAND_OR_TOWN_CARD: Selection = Selection::of(select::up_to(1), &[select::basic_land_or_subtype(Subtype::Town)]).called("basic land or Town");

/// "Search your library for a basic land card or Town card, put it onto the battlefield tapped, then shuffle. When you search
/// your library this way, put a +1/+1 counter on target creature you control." (CR 701.23, 701.24, 603.12): the card found
/// enters tapped before the shuffle; the reflexive ability is the card's delayed ability 0, triggered by the search itself
/// whether or not a card was found, and chooses its target as it goes on the stack, once this spell has finished resolving.
fn search_then_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, BASIC_LAND_OR_TOWN_CARD, true)?;
    let this = g.card_id(&CARD);
    g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], search_then_counter),
    delayed: &[delayed_trigger(
        effects::plus_one_counter_on_target,
        "When you search your library this way, put a +1/+1 counter on target creature you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::PRISHES_WANDERINGS
};
