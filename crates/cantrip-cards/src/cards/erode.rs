use cantrip_api::*;
use cantrip_mechanics::library::{search_to_battlefield, Find};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Destroy target creature or planeswalker. Its controller (the one it last had, CR 608.2h) may search their library
/// for a basic land card, put it onto the battlefield tapped, then shuffle; that happens even when the destruction
/// did not (it was indestructible), as long as the target was still legal.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = cantrip_mechanics::targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.controller(permanent);
    g.destroy(permanent);
    if !cantrip_mechanics::effects::ask_may(g, owner, item.object, Purpose::Optional)? {
        return Ok(());
    }
    search_to_battlefield(g, owner, item.object, Find::ONE, true, |q, c| {
        let chars = q.chars(c);
        chars.is(CardType::Land) && chars.supertypes.basic
    })?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::CREATURE_OR_PLANESWALKER], effect }),
    ..oracle::ERODE
};
