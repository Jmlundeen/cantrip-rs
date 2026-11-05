use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Put two +1/+1 counters and a reach counter on target creature" (CR 122.1b): nothing if the target is no longer legal
/// (CR 608.2b).
fn renew(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 2);
    g.add_counters(creature, CounterKind::Reach, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // Reach is a core keyword (CR 702.17), read from the card's JSON. Renew is an ability word (CR 207.2c): the ability is an
    // ordinary activated ability of the card in the graveyard.
    abilities: &[ability(cost("{4}{G}").and(EXILE_THIS), renew, "Renew \u{2014} {4}{G}, Exile this card from your graveyard: Put two +1/+1 counters and a reach counter on target creature. Activate only as a sorcery.")
        .from(ZoneKind::Graveyard)
        .sorcery_speed()
        .targets(&[targets::CREATURE])],
    ..oracle::SAGU_PUMMELER
};
