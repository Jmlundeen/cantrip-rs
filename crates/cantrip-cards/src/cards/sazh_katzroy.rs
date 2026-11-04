use cantrip_api::*;
use cantrip_mechanics::library::{ask_may_search, find_to_hand};
use cantrip_mechanics::select::{self, Selection};
use cantrip_mechanics::triggers::{THIS_ATTACKS, THIS_ENTERS};
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("SearchLibrary", "search your library for a Bird or basic land card?", "Reveal it and put it into your hand, then shuffle", "Search", "Don't search");

/// "A Bird or basic land card", which may be left unfound (CR 701.23b): a card with the subtype Bird, or a basic land card.
const BIRD_OR_BASIC_LAND_CARD: Selection = Selection::of(select::up_to(1), &[select::basic_land_or_subtype(Subtype::Bird)]).called("Bird or basic land");

/// "When this creature enters, you may search your library for a Bird or basic land card, reveal it, put it into your hand, then
/// shuffle" (CR 701.23): asked first, and a declined search shuffles nothing.
fn search_for_a_bird_or_basic_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if ask_may_search(g, item.controller, item.object, &ASK)? {
        find_to_hand(g, item.controller, item.object, BIRD_OR_BASIC_LAND_CARD, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            search_for_a_bird_or_basic_land,
            "When this creature enters, you may search your library for a Bird or basic land card, reveal it, put it into your hand, then shuffle.",
        ),
        triggered(
            THIS_ATTACKS,
            effects::plus_one_counter_on_target_then_double,
            "Whenever this creature attacks, put a +1/+1 counter on target creature, then double the number of +1/+1 counters on that creature.",
        )
        .targets(&[targets::CREATURE]),
    ],
    ..oracle::SAZH_KATZROY
};
