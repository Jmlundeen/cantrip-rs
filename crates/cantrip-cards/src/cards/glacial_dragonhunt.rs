use cantrip_api::*;
use cantrip_mechanics::{choose, damage_target, hand, harmonize, targets};
use cantrip_oracle as oracle;

/// "Draw a card, then you may discard a card. When you discard a nonland card this way, Glacial Dragonhunt deals 3 damage to
/// target creature." The discard is a choice made as the spell resolves (CR 608.2d); declining is the empty answer to an
/// up-to-one choice, and an empty hand offers none. The reflexive ability (CR 603.12) triggers only when a nonland card was
/// discarded - one put into a hidden zone unrevealed is undefined, no nonland card (CR 701.9c, `hand::discarded_nonland_card`)
/// - and its target is chosen as it goes on the stack.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    g.draw(you, 1);
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let chosen = choose::cards(g, you, item.object, Purpose::Discard, hand, 1, true)?;
    let discarded = g.discard_together(&chosen);
    if discarded.iter().any(|&card| hand::discarded_nonland_card(&*g, card)) {
        let this = g.card_id(&CARD);
        g.reflexive_trigger(item.object, you, AbilityRef::delayed(this, 0), Target::None);
    }
    Ok(())
}

/// "Glacial Dragonhunt deals 3 damage to target creature": the spell, as it last existed, is the source (CR 608.2h).
fn damage_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    delayed: &[delayed_trigger(damage_creature, "When you discard a nonland card this way, Glacial Dragonhunt deals 3 damage to target creature.")
        .targets(&[targets::CREATURE])],
    ways: &[harmonize::way(mana("{4}{U}{R}"), "Harmonize {4}{U}{R}")],
    ..oracle::GLACIAL_DRAGONHUNT
};
