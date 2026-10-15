use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::{deals_combat_damage_to_player, this_enters};
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
        AbilityDef::Triggered(TriggeredDef {
            text: "When Ivora enters and whenever it deals combat damage to a player, create a Blood token.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: blood,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When Ivora enters and whenever it deals combat damage to a player, create a Blood token.",
            listens: EventMask::DAMAGE,
            matches: deals_combat_damage_to_player,
            effect: blood,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever you discard a card, put a +1/+1 counter on Ivora.",
            listens: EventMask::DISCARDED,
            matches: you_discard,
            effect: counter,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::IVORA_INSATIABLE_HEIR
};

pub static EXTRA: &[&CardDef] = &[&tokens::BLOOD];
