use cantrip_api::*;
use cantrip_mechanics::library::{search_to_battlefield, Find};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::web_slinging;
use cantrip_oracle as oracle;

/// "Search your library for a basic land card, put it onto the battlefield tapped, then shuffle." (CR 701.23)
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    search_to_battlefield(g, item.controller, item.object, Find::ONE, true, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    ways: &[web_slinging::way(mana("{2}{G}"), "Web-slinging {2}{G}")],
    abilities: &[triggered(
        THIS_ENTERS,
        search,
        "When Spider-Man enters, search your library for a basic land card, put it onto the battlefield tapped, then shuffle.",
    )],
    ..oracle::SPIDER_MAN_BROOKLYN_VISIONARY
};
