use cantrip_api::*;
use cantrip_mechanics::triggers::{of_subtype_you_control_dies, BEGINNING_OF_YOUR_COMBAT};
use cantrip_mechanics::{lands, targets};
use cantrip_oracle as oracle;

/// "Put a number of +1/+1 counters equal to Jenova's power on up to one other target creature. That creature becomes a Mutant
/// in addition to its other types" (CR 122.1, 205.1b, 613.1d): Jenova's power as it last existed on the battlefield if it has
/// left (CR 608.2h), none for a power below one. The type is a continuous effect with no end, read through the words of
/// Jenova's text (CR 612). Nothing if the target is no longer legal (CR 608.2b).
fn mutate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let counters = targets::power_or_last_known(g, item.object).max(0);
    g.add_counters(creature, CounterKind::PlusOne, counters);
    let mutant = g.words(item.object).subtype(Subtype::Mutant);
    g.add_effect(item.object, creature, EffectKind::AddSubtype(mutant), Duration::Indefinite);
    Ok(())
}

/// "Whenever a Mutant you control dies during your turn" (CR 700.4, 603.10a): a Mutant as it last was on the battlefield, while
/// its controller (this ability's) is the active player. The ability is about the creature that died.
fn mutant_you_control_dies_during_your_turn(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if q.state().turn.active != me.controller {
        return None;
    }
    of_subtype_you_control_dies::<{ lands::sub(Subtype::Mutant) }>(q, me, event)?;
    let Event::ZoneChange { old, .. } = event else { return None };
    Fired::about(Target::Object(*old))
}

/// "You draw cards equal to its power" (CR 121.1, 608.2h): the power the Mutant last had on the battlefield; none for a power
/// below one.
fn draw_its_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(mutant) = item.subject.object() else { return Ok(()) };
    let n = u32::try_from(targets::power_or_last_known(g, mutant)).unwrap_or_default();
    g.draw(item.controller, n);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            BEGINNING_OF_YOUR_COMBAT,
            mutate,
            "At the beginning of combat on your turn, put a number of +1/+1 counters equal to Jenova's power on up to one other target creature. That creature becomes a Mutant in addition to its other types.",
        )
        .targets(&[targets::OTHER_CREATURE.optional()]),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, mutant_you_control_dies_during_your_turn),
            draw_its_power,
            "Whenever a Mutant you control dies during your turn, you draw cards equal to its power.",
        ),
    ],
    ..oracle::JENOVA_ANCIENT_CALAMITY
};
