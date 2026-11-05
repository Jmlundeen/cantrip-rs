use cantrip_api::*;
use cantrip_mechanics::triggers::{another_nontoken_you_control_enters, CREATURE_TYPE, YOUR_END_STEP};
use cantrip_mechanics::{effects, endure};
use cantrip_oracle as oracle;

/// "Whenever another nontoken creature you control enters, it endures X, where X is the number of counters on this creature."
/// (CR 701.63a, 603.6a): the creature that entered endures, and the choice is made as the ability resolves. X is read then, counters
/// of every kind, as this creature last existed if it has left the battlefield (CR 608.2h); with none, endure 0 does nothing.
fn it_endures_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(entered) = item.subject else { return Ok(()) };
    let x = effects::counters_on(g, item.object);
    endure::endure(g, entered, item.object, x as i32)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(YOUR_END_STEP, effects::plus_one_counter_on_this, "At the beginning of your end step, put a +1/+1 counter on this creature."),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, another_nontoken_you_control_enters::<CREATURE_TYPE>),
            it_endures_x,
            "Whenever another nontoken creature you control enters, it endures X, where X is the number of counters on this creature. (Put X +1/+1 counters on the creature that entered or create an X/X white Spirit creature token.)",
        ),
    ],
    ..oracle::WARDEN_OF_THE_GROVE
};

pub static EXTRA: &[&CardDef] = &[&endure::SPIRIT];
