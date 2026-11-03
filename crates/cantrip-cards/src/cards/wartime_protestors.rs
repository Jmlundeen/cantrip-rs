use cantrip_api::*;
use cantrip_mechanics::effects::gain_until_end_of_turn;
use cantrip_mechanics::{lands, targets, triggers};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on that creature and it gains haste until end of turn": the creature that entered, the trigger's
/// subject. If it has left the battlefield by now it is a new object (CR 400.7) and neither happens (CR 608.2b).
fn counter_and_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = item.subject.object().filter(|&c| targets::permanent(g, c)) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    gain_until_end_of_turn(g, item.object, creature, Keyword::Haste);
    Ok(())
}

/// Haste is a core keyword read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Ally) }>),
        counter_and_haste,
        "Whenever another Ally you control enters, put a +1/+1 counter on that creature and it gains haste until end of turn.",
    )],
    ..oracle::WARTIME_PROTESTORS
};
