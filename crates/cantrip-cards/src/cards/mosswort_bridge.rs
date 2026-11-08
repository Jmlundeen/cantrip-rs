use cantrip_api::*;
use cantrip_mechanics::hideaway;
use cantrip_mechanics::hideaway_play::play_exiled_card;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{creatures_have_total_power_at_least, mana};
use cantrip_oracle as oracle;

/// "Hideaway 4" (CR 702.75a): the shared hideaway-four look, exile and bottoming as this land enters.
fn hide_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hideaway(g, item, 4)
}

/// "You may play the exiled card without paying its mana cost if creatures you control have total power 10 or greater"
/// (CR 118.9, 608.2g): the power of your creatures only, checked once as the ability resolves.
fn play_the_exiled_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !creatures_have_total_power_at_least(g, item.controller, 10) {
        return Ok(());
    }
    play_exiled_card(g, item, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        triggered(
            THIS_ENTERS,
            hide_a_card,
            "Hideaway 4 (When this land enters, look at the top four cards of your library, exile one face down, then put the rest on the bottom in a random order.)",
        ),
        ability(
            cost("{G}, {T}"),
            play_the_exiled_card,
            "{G}, {T}: You may play the exiled card without paying its mana cost if creatures you control have total power 10 or greater.",
        ),
    ],
    enters: Some(mana::enters_tapped),
    ..oracle::MOSSWORT_BRIDGE
};
