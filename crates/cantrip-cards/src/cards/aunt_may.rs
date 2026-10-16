use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You gain 1 life. If it's a Spider, put a +1/+1 counter on it." The entering creature is the trigger's subject; if it
/// has left the battlefield by now it is a new object (CR 400.7), so the counter goes nowhere, but the life is gained regardless. Its
/// type is read as it is now, through the words of Aunt May's own text (CR 612).
fn welcome(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.gain_life(item.controller, 1);
    if let Target::Object(creature) = item.subject
        && g.chars(creature).has_subtype(g.words(item.object).subtype(Subtype::Spider))
    {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::ZONE_CHANGE, cantrip_mechanics::triggers::another_creature_you_control_enters),
        welcome,
        "Whenever another creature you control enters, you gain 1 life. If it's a Spider, put a +1/+1 counter on it.",
    )],
    ..oracle::AUNT_MAY
};
