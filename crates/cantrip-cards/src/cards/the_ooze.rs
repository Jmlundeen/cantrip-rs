use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Whenever a creature you control with a +1/+1 counter on it leaves the battlefield" (CR 603.6c, 603.10a): looked back
/// on, so the creature is judged as it last was on the battlefield: a creature then, under the controller it last had, with
/// its +1/+1 counters. The trigger carries how many it had ("for each +1/+1 counter on it").
fn creature_with_a_counter_leaves(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, .. } if from.is(ZoneKind::Battlefield) => {
            let counters = q.obj(*old).counters(CounterKind::PlusOne);
            if counters > 0 && q.chars(*old).is(CardType::Creature) && q.obj(*old).controller == me.controller {
                Some(Fired { subject: Target::None, data: [counters as u32, 0] })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// "Create a Mutagen token for each +1/+1 counter on it."
fn mutagens_for_the_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::MUTAGEN, item.data[0])?;
    Ok(())
}

/// "{T}: Exile target card from a graveyard. Create a Mutagen token." With the target illegal the ability does not
/// resolve (CR 608.2b), so no token is made.
fn exile_and_mutagen(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    g.exile(card);
    tokens::create(g, item.controller, &tokens::MUTAGEN, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, creature_with_a_counter_leaves),
            mutagens_for_the_counters,
            "Whenever a creature you control with a +1/+1 counter on it leaves the battlefield, create a Mutagen token for each +1/+1 counter on it.",
        )
        .look_back(),
        ability(TAP, exile_and_mutagen, "{T}: Exile target card from a graveyard. Create a Mutagen token.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD]),
    ],
    ..oracle::THE_OOZE
};

pub static EXTRA: &[&CardDef] = &[&tokens::MUTAGEN];
