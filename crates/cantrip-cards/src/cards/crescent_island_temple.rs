use cantrip_api::*;
use cantrip_mechanics::tokens::{self, RED_MONK};
use cantrip_mechanics::{lands, permanents_with_subtype_you_control, triggers};
use cantrip_oracle as oracle;

/// "For each Shrine you control, create a 1/1 red Monk creature token with prowess" (CR 608.2h): the Shrines are counted as the
/// ability resolves, through the words of this card's text, the Temple among them while it is on the battlefield.
fn monk_for_each_shrine(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let shrine = g.words(item.object).subtype(Subtype::Shrine);
    let n = permanents_with_subtype_you_control(g, item.controller, shrine) as u32;
    tokens::create(g, item.controller, &RED_MONK, n)?;
    Ok(())
}

fn make_monk(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &RED_MONK, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            monk_for_each_shrine,
            "When Crescent Island Temple enters, for each Shrine you control, create a 1/1 red Monk creature token with prowess.",
        ),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, triggers::another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Shrine) }>),
            make_monk,
            "Whenever another Shrine you control enters, create a 1/1 red Monk creature token with prowess.",
        ),
    ],
    ..oracle::CRESCENT_ISLAND_TEMPLE
};

pub static EXTRA: &[&CardDef] = &[&RED_MONK];
