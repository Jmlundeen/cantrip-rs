use cantrip_api::*;
use cantrip_mechanics::hideaway;
use cantrip_mechanics::hideaway_play::play_exiled_card;
use cantrip_mechanics::mana::ISLAND;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn hide_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hideaway(g, item.controller, item.object, 4)
}

/// "You may play the exiled card without paying its mana cost if a library has twenty or fewer cards in it": no player is named,
/// so every library in the game is asked, checked once as the ability resolves.
fn play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let small = g.state().apnap().any(|p| g.state().zone(Zone::library(p)).len() <= 20);
    if small {
        play_exiled_card(g, item.object, item.controller)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ISLAND,
        AbilityDef::Triggered(TriggeredDef {
            text: "Hideaway 4 (When this land enters, look at the top four cards of your library, exile one face down, then put the rest on the bottom in a random order.)",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: hide_a_card,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{U}, {T}: You may play the exiled card without paying its mana cost if a library has twenty or fewer cards in it.",
            cost: Cost { mana: mana(0, 0, 1, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: play,
            ..ActivatedDef::BLANK
        }),
    ],
    enters: Some(cantrip_mechanics::mana::enters_tapped),
    ..oracle::SHELLDOCK_ISLE
};
