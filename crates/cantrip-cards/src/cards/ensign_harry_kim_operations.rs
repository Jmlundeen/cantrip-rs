use cantrip_api::*;
use cantrip_mechanics::triggers::another_creature_you_control_enters;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// A permanent's power as the game sees it now, or as it last had it once it has left (CR 608.2h).
fn power(q: &dyn Query, id: ObjectId) -> i16 {
    if targets::permanent(q, id) {
        q.chars(id).power.unwrap_or(0)
    } else {
        q.state().last_known(id).map_or(0, |known| known.power.unwrap_or(0))
    }
}

/// "Whenever another creature you control enters, if it has greater power than Ensign Kim" (CR 603.4): the entering
/// creature's power as it enters is compared with Ensign Kim's as it is.
fn bigger_creature_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let fired = another_creature_you_control_enters(q, me, event)?;
    let Target::Object(creature) = fired.subject else { return None };
    (power(q, creature) > power(q, me.source)).then_some(fired)
}

/// The intervening "if" is true again as the ability resolves (CR 603.4): a creature that has shrunk by then is no
/// longer greater, and the counter is not put on.
fn put_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject
        && power(g, creature) > power(g, item.object)
    {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// "{2}, Remove two +1/+1 counters from among creatures you control: Draw a card." (CR 118.3, 122.1): only +1/+1
/// counters pay it, from among the creatures their controller controls.
fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw(g, item, 1)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, bigger_creature_enters),
            put_counter,
            "Whenever another creature you control enters, if it has greater power than Ensign Kim, put a +1/+1 counter on Ensign Kim.",
        ),
        ability(
            cost("{2}").and(remove_kind_counters_among(CounterKind::PlusOne, 2, targets::filters::creature)),
            draw,
            "{2}, Remove two +1/+1 counters from among creatures you control: Draw a card.",
        ),
    ],
    ..oracle::ENSIGN_HARRY_KIM_OPERATIONS
};
