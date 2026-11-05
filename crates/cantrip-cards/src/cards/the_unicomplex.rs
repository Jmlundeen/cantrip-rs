use cantrip_api::*;
use cantrip_mechanics::{effects, mana, targets, triggers::THIS_ENTERS_UNTAPPED};
use cantrip_oracle as oracle;

/// "Put target creature card from your graveyard on top of your library." (CR 401.4): the target, if it is still a legal
/// target as the ability resolves (CR 608.2b), to the top of its owner's library.
fn put_on_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.put_into_library(&[card], LibraryPlace::Top);
    Ok(())
}

/// "When this land enters untapped, choose one —" (CR 603.6a, 700.2b).
static MODAL: ModalDef = modes(&[
    mode(put_on_top, "Put target creature card from your graveyard on top of your library.")
        .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD]),
    mode(|g, item| effects::surveil(g, item, 1), "Surveil 1."),
]);

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped_unless_two_or_more_other_lands),
    abilities: &[
        modal_triggered(
            THIS_ENTERS_UNTAPPED,
            &MODAL,
            "When this land enters untapped, choose one —\n• Put target creature card from your graveyard on top of your library.\n• Surveil 1.",
        ),
        mana_ability(TAP, add(Mana::B), "{T}: Add {B}."),
    ],
    ..oracle::THE_UNICOMPLEX
};
