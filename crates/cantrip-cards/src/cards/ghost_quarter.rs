use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::library::{search_to_battlefield, Find};
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

/// Destroy target land. Its controller (the one it last had, CR 608.2h) may search their library for a basic land
/// card, put it onto the battlefield, then shuffle; declining skips the search and the shuffle. Nothing happens at
/// all if the land is no longer a legal target.
fn destroy_and_search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(land) = cantrip_mechanics::targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.controller(land);
    g.destroy(land);
    if !cantrip_mechanics::effects::ask_may(g, owner, item.object, Purpose::Optional)? {
        return Ok(());
    }
    search_to_battlefield(g, owner, item.object, Find::ONE, false, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana::tap_for(mana::add_c, Mana::C, "{T}: Add {C}."),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice this land: Destroy target land. Its controller may search their library for a basic land card, put it onto the battlefield, then shuffle.",
            cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
            targets: &[targets::LAND],
            effect: destroy_and_search,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::GHOST_QUARTER
};
