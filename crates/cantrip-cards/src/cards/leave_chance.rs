use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// "Target permanents you own" (CR 108.3): ownership, not control.
fn permanent_you_own(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if cantrip_mechanics::targets::permanent(q, id) && q.obj(id).owner == asker.controller)
}

/// Leave: "Return any number of target permanents you own to your hand."
fn leave(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.return_to_hand(&cantrip_mechanics::targets::legal_objects(g, item));
    Ok(())
}

/// Chance: "Discard any number of cards, then draw that many cards." The discard is one event (CR 701.9).
fn chance(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let hand: Vec<ObjectId> = g.state().zone(Zone::hand(you)).to_vec();
    let cards = choose::cards(g, you, item.object, Purpose::Discard, hand, u8::MAX, true)?;
    if cards.is_empty() {
        return Ok(());
    }
    g.discard_together(&cards);
    g.draw(you, cards.len() as u32);
    Ok(())
}

pub static LEAVE: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 0, max: 16, filter: permanent_you_own, label: "" }], effect: leave }),
    ..oracle::LEAVE
};

pub static CHANCE: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect: chance }), ..oracle::CHANCE };

pub static CARD: CardDef = CardDef { split: Some((&LEAVE, &CHANCE)), ..oracle::LEAVE_CHANCE };

pub static EXTRA: &[&CardDef] = &[&LEAVE, &CHANCE];
