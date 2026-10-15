use cantrip_api::*;
use cantrip_mechanics::hideaway;
use cantrip_mechanics::hideaway_play::play_exiled_card;
use cantrip_mechanics::mana::PLAINS;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn hide_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hideaway(g, item.controller, item.object, 4)
}

/// How many different creatures the player declared as attackers this turn (CR 508.1): one that attacked in two combats
/// counts once, and one put onto the battlefield attacking was never declared.
fn attacked_with(q: &dyn Query, player: PlayerId) -> usize {
    let mut seen: Vec<ObjectId> = Vec::new();
    for r in q.history().this_turn() {
        if let Event::Attacks { attacker, .. } = r.event
            && q.obj(attacker).controller == player
            && !seen.contains(&attacker)
        {
            seen.push(attacker);
        }
    }
    seen.len()
}

/// "You may play the exiled card without paying its mana cost if you attacked with three or more creatures this turn"
/// (CR 118.9, 508.1, 608.2g), the card found by the exile link off this permanent as it was.
fn play_the_exiled_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    if attacked_with(g, you) < 3 {
        return Ok(());
    }
    play_exiled_card(g, item.object, you)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        PLAINS,
        AbilityDef::Triggered(TriggeredDef {
            text: "Hideaway 4 (When this land enters, look at the top four cards of your library, exile one face down, then put the rest on the bottom of your library in a random order.)",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: hide_a_card,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{W}, {T}: You may play the exiled card without paying its mana cost if you attacked with three or more creatures this turn.",
            cost: Cost { mana: mana(0, 1, 0, 0, 0, 0), tap: true, ..Cost::NONE },
            effect: play_the_exiled_card,
            ..ActivatedDef::BLANK
        }),
    ],
    enters: Some(cantrip_mechanics::mana::enters_tapped),
    ..oracle::WINDBRISK_HEIGHTS
};
