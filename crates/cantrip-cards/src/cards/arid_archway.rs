use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, lands, mana};
use cantrip_oracle as oracle;

/// "When this land enters, return a land you control to its owner's hand. If another Desert was returned this way, surveil
/// 1." (CR 603.6a, 701.25.) The land is judged as it is on the battlefield, before it is returned, and the surveil happens
/// only if it was returned: not for the Archway itself, for a land of another type, or for a land that left in the meantime.
fn bounce_and_surveil(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(land) = lands::choose_a_land_to_return(g, item)? else { return Ok(()) };
    let desert = land != item.object && g.chars(land).has_subtype(g.words(item.object).subtype(Subtype::Desert));
    let returned = !g.return_to_hand(&[land]).is_empty();
    if desert && returned {
        effects::surveil(g, item, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}."),
        triggered(
            THIS_ENTERS,
            bounce_and_surveil,
            "When this land enters, return a land you control to its owner's hand. If another Desert was returned this way, surveil 1.",
        ),
    ],
    ..oracle::ARID_ARCHWAY
};
