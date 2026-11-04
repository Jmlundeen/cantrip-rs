use cantrip_api::*;
use cantrip_mechanics::{crew::crew, permanents_with_subtype_you_control, pump, triggers::THIS_ATTACKS_OR_BLOCKS};
use cantrip_oracle as oracle;

/// "Whenever this Vehicle attacks or blocks, it gets +1/+1 until end of turn for each Spider you control" (CR 611.2a): the Spiders
/// counted as this resolves, whatever their card types (a Spider is a creature type, 205.3m), the word read through this
/// Vehicle's own text (CR 612). Nothing happens if it is no longer on the battlefield.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let spider = g.words(item.object).subtype(Subtype::Spider);
    let spiders = permanents_with_subtype_you_control(g, item.controller, spider) as i16;
    pump(g, item.object, item.object, spiders, spiders);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS_OR_BLOCKS, grow, "Whenever this Vehicle attacks or blocks, it gets +1/+1 until end of turn for each Spider you control."),
        crew(2, "Crew 2 (Tap any number of creatures you control with total power 2 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::SPIDER_MOBILE
};
