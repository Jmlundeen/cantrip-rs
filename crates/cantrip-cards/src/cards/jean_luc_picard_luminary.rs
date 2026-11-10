use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::{self, YOU_DRAW_YOUR_SECOND_CARD};
use cantrip_oracle as oracle;

/// "Whenever another historic permanent you control with mana value 3 or greater enters" (CR 603.6a, 700.6, 202.3): another
/// permanent under Jean-Luc's controller's control as it enters, historic — an artifact, a legendary permanent, or a Saga —
/// with a mana value of 3 or more. Jean-Luc himself is the source, never "another".
fn another_big_historic_permanent_you_control_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let new = triggers::another_permanent_you_control_enters_with_mana_value_at_least(q, me, event, 3)?;
    if triggers::is_historic(q, new) { Fired::PLAIN } else { None }
}

fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

/// "up to one target Spacecraft" (CR 115.1, 205.3g): a permanent with the Spacecraft subtype, whether or not it is a creature
/// right now (a Spacecraft is one only while its charge counters station it at 8+).
fn spacecraft_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_of_any_subtype(q, asker, t, &[Subtype::Spacecraft])
}

/// "put a +1/+1 counter on up to one other target creature and two charge counters on up to one target Spacecraft" (CR 122.6,
/// 608.2b): one counter on the creature, two on the Spacecraft, each instruction skipped when its target was not chosen or is
/// no longer legal as the ability resolves.
fn a_counter_and_two_charge_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    if let Some(spacecraft) = targets::object(g, item, 1) {
        g.add_counters(spacecraft, CounterKind::Charge, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, another_big_historic_permanent_you_control_enters),
            draw_a_card,
            "Whenever another historic permanent you control with mana value 3 or greater enters, draw a card.",
        ),
        triggered(
            YOU_DRAW_YOUR_SECOND_CARD,
            a_counter_and_two_charge_counters,
            "Seek out \u{2014} Whenever you draw your second card each turn, put a +1/+1 counter on up to one other target creature and two charge counters on up to one target Spacecraft.",
        )
        .targets(&[up_to_one(targets::other_creature_filter), up_to_one(spacecraft_filter)]),
    ],
    ..oracle::JEAN_LUC_PICARD_LUMINARY
};
