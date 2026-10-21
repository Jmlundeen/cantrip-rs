use cantrip_api::*;
use cantrip_mechanics::{choose, pump, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// You may discard any number of cards, together (CR 701.9). When you do (CR 603.12), the reflexive ability triggers
/// counting them: "that many" is the number of cards discarded, and none discarded is not doing it.
fn discard_any(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let hand = g.state().zone(Zone::hand(you)).to_vec();
    let chosen = choose::any_number_of(g, you, item.object, Purpose::Discard, "", hand.clone(), hand)?;
    let discarded = g.discard_together(&chosen);
    if !discarded.is_empty() {
        let this = g.card_id(&CARD);
        g.reflexive_trigger_counted(item.object, you, AbilityRef::delayed(this, 0), Target::None, discarded.len() as u16);
    }
    Ok(())
}

/// Up to that many target creatures each get -2/-2 until end of turn: each target still legal (CR 608.2b).
fn shrink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for creature in targets::legal_objects(g, item) {
        pump(g, item.object, creature, -2, -2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, discard_any, "When this creature enters, you may discard any number of cards. When you do, up to that many target creatures each get -2/-2 until end of turn.")],
    delayed: &[delayed_trigger(shrink, "When you do, up to that many target creatures each get -2/-2 until end of turn.")
        .targets(&[targets::CREATURE.between(0, TargetSpec::COUNTED)])],
    ..oracle::MIASMA_DEMON
};
