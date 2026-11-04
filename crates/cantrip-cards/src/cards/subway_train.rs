use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::{lands, triggers};
use cantrip_oracle as oracle;

const PAY: Question = Question::new("Pay", "pay {cost} to search for a basic land card?", "", "Pay {cost}", "Don't pay");

/// "When this Vehicle enters, you may pay {G}. If you do, search your library for a basic land card, reveal it, put it into
/// your hand, then shuffle" (CR 603.6a, 118.12, 701.23): the search and the shuffle happen only when {G} was paid.
fn pay_to_fetch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.may_pay(item.controller, item.object, item.object, cost("{G}"), &PAY)? {
        lands::search_basic_land_to_hand(g, item.controller, item.object)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            pay_to_fetch,
            "When this Vehicle enters, you may pay {G}. If you do, search your library for a basic land card, reveal it, put it into your hand, then shuffle.",
        ),
        crew(2, "Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::SUBWAY_TRAIN
};
