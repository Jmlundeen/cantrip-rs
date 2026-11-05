use cantrip_api::*;
use cantrip_mechanics::hideaway;
use cantrip_mechanics::hideaway_play::play_exiled_card;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn hide_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    hideaway(g, item, 4)
}

/// "You may play the exiled card without paying its mana cost if you control four or more legendary creatures" (CR 118.9, 608.2g),
/// counted as the ability resolves, the card found by the exile link off this permanent as it was.
fn play_the_exiled_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let legendary = g
        .permanents_of(you)
        .into_iter()
        .filter(|&id| {
            let c = g.chars(id);
            c.is(CardType::Creature) && c.supertypes.legendary
        })
        .count();
    if legendary < 4 {
        return Ok(());
    }
    play_exiled_card(g, item, you)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, hide_a_card, "Hideaway 4 (When this land enters, look at the top four cards of your library, exile one face down, then put the rest on the bottom in a random order.)"),
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}").and(TAP), play_the_exiled_card, "{2}, {T}: You may play the exiled card without paying its mana cost if you control four or more legendary creatures."),
    ],
    ..oracle::CLIVES_HIDEAWAY
};
