use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::lands::{count_you_control, may_fetch_basic};
use cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS;
use cantrip_oracle as oracle;

/// "This Vehicle's power is equal to the number of lands you control" (CR 604.3, layer 7a): a characteristic-defining
/// ability, so it works in every zone, counting its owner's lands off the battlefield.
fn power_is_lands_you_control(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    chars.power = Some(count_you_control(q, source, &|c| c.is(CardType::Land)));
}

/// "Whenever this Vehicle enters or attacks, you may search your library for a basic land card, put it onto the battlefield
/// tapped, then shuffle" (CR 701.23): declining skips the search and the shuffle.
fn fetch_a_basic(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    may_fetch_basic(g, item.controller, item.object, true)
}

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(Layer::PtDefine, power_is_lands_you_control, "This Vehicle's power is equal to the number of lands you control.").lost_pt(0, 4)],
    abilities: &[
        triggered(
            THIS_ENTERS_OR_ATTACKS,
            fetch_a_basic,
            "Whenever this Vehicle enters or attacks, you may search your library for a basic land card, put it onto the battlefield tapped, then shuffle.",
        ),
        crew(4, "Crew 4 (Tap any number of creatures you control with total power 4 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::LUMBERING_WORLDWAGON
};
