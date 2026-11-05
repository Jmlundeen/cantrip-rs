use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter and a trample counter on target creature" (CR 122.1b): nothing if the target is no longer legal
/// (CR 608.2b).
fn renew(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    g.add_counters(creature, CounterKind::Trample, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // Trample is a core keyword (CR 702.19), read from the card's JSON. Renew is an ability word (CR 207.2c): the ability is
    // an ordinary activated ability of the card in the graveyard.
    abilities: &[ability(cost("{1}{G}").and(EXILE_THIS), renew, "Renew \u{2014} {1}{G}, Exile this card from your graveyard: Put a +1/+1 counter and a trample counter on target creature. Activate only as a sorcery.")
        .from(ZoneKind::Graveyard)
        .sorcery_speed()
        .targets(&[targets::CREATURE])],
    ..oracle::CHAMPION_OF_DUSAN
};
