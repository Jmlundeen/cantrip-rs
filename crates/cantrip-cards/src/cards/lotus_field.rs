use cantrip_api::*;
use cantrip_mechanics::{mana, sacrifice_chosen, triggers};
use cantrip_oracle as oracle;

/// "When this land enters, sacrifice two lands." (CR 701.21a): its controller chooses two lands they control, this one
/// included, or all of them when they control fewer than two; the two go at once.
fn sacrifice_two_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    sacrifice_chosen(g, item.controller, item.object, 2, |c| c.is(CardType::Land)).map(drop)
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        triggered(triggers::THIS_ENTERS, sacrifice_two_lands, "When this land enters, sacrifice two lands."),
        mana_ability(TAP, add_any_color(3), "{T}: Add three mana of any one color."),
    ],
    ..oracle::LOTUS_FIELD
};
