use cantrip_api::*;
use cantrip_mechanics::{permanents_with_subtype_you_control, targets};
use cantrip_oracle as oracle;

/// "Put target creature on the bottom of its owner's library. You lose 2 life unless you control a Villain." (CR 401.4, 608.2c):
/// the Villain is looked for as the second sentence resolves, after the creature has left, so a Villain you control that was the
/// target no longer counts; a spell whose only target has become illegal does nothing at all (CR 608.2b).
fn bottom_then_lose_unless_villain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_into_library(&[creature], LibraryPlace::Bottom);
    g.end_sentence();
    let villain = g.words(item.object).subtype(Subtype::Villain);
    if permanents_with_subtype_you_control(g, item.controller, villain) == 0 {
        g.lose_life(item.controller, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], bottom_then_lose_unless_villain), ..oracle::THE_SPOTS_PORTAL };
