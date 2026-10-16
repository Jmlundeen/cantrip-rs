use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{DEALS_COMBAT_DAMAGE_TO_PLAYER, THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Create a Blood token."
fn blood(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_blood(g, item.controller)?;
    Ok(())
}

/// "Whenever you discard a card" (CR 701.9a): each card of a discard is its own trigger.
fn you_discard(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Discarded { player, .. } if *player == q.controller(source) => Fired::PLAIN,
        _ => None,
    }
}

fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, blood, "When Ivora enters and whenever it deals combat damage to a player, create a Blood token."),
        triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER, blood, "When Ivora enters and whenever it deals combat damage to a player, create a Blood token."),
        triggered(when(EventMask::DISCARDED, you_discard), counter, "Whenever you discard a card, put a +1/+1 counter on Ivora."),
    ],
    ..oracle::IVORA_INSATIABLE_HEIR
};

pub static EXTRA: &[&CardDef] = &[&tokens::BLOOD];
