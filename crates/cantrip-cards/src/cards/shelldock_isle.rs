use cantrip_api::*;
use cantrip_mechanics::hideaway;
use cantrip_mechanics::hideaway_play::play_exiled_card;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn hide_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hideaway(g, item, 4)
}

/// "You may play the exiled card without paying its mana cost if a library has twenty or fewer cards in it": no player is named,
/// so every library in the game is asked, checked once as the ability resolves.
fn play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let small = g.state().apnap().any(|p| g.state().zone(Zone::library(p)).len() <= 20);
    if small {
        play_exiled_card(g, item, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::U), "{T}: Add {U}."),
        triggered(THIS_ENTERS, hide_a_card, "Hideaway 4 (When this land enters, look at the top four cards of your library, exile one face down, then put the rest on the bottom in a random order.)"),
        ability(cost("{U}, {T}"), play, "{U}, {T}: You may play the exiled card without paying its mana cost if a library has twenty or fewer cards in it."),
    ],
    enters: Some(cantrip_mechanics::mana::enters_tapped),
    ..oracle::SHELLDOCK_ISLE
};
