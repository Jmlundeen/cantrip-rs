use cantrip_api::*;
use cantrip_mechanics::{effects, tokens};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

use crate::lingering_souls::SPIRIT;

fn spirit(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &SPIRIT, 1)?;
    Ok(())
}

/// "Whenever you create one or more creature tokens" (CR 111.2, 603.2c): a creation, not an entry, so a token that
/// became one as a copy of a permanent spell does not count (CR 111.13). "You create" is the creator, the token's owner.
/// One counter for the tokens one creation made.
fn you_create_creature_tokens(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { new, reason: ZoneReason::Created, .. }
            if q.obj(*new).owner == q.controller(source) && q.chars(*new).is(CardType::Creature) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

fn story_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Story, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this artifact enters, create a 1/1 white Spirit creature token with flying.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: spirit,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever you create one or more creature tokens, put a story counter on this artifact.",
            listens: EventMask::ZONE_CHANGE,
            matches: you_create_creature_tokens,
            effect: story_counter,
            once_per_batch: true,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{W}, {T}, Remove a story counter from this artifact: Draw a card.",
            cost: Cost {
                mana: mana(0, 1, 0, 0, 0, 0),
                tap: true,
                remove_counters: Some((CounterKind::Story, 1)),
                ..Cost::NONE
            },
            effect: effects::draw::<1>,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::STAFF_OF_THE_STORYTELLER
};
