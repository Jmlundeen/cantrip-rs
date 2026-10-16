use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Put a flying counter, a deathtouch counter, and a lifelink counter on target creature" (CR 122.1b): nothing if the target
/// is no longer legal (CR 608.2b).
fn renew(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::Flying, 1);
    g.add_counters(creature, CounterKind::Deathtouch, 1);
    g.add_counters(creature, CounterKind::Lifelink, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // Renew is an ability word (CR 207.2c): the ability is an ordinary activated ability of the card in the graveyard.
    abilities: &[ability(cost("{2}{B}").and(EXILE_THIS), renew, "Renew \u{2014} {2}{B}, Exile this card from your graveyard: Put a flying counter, a deathtouch counter, and a lifelink counter on target creature. Activate only as a sorcery.")
        .from(ZoneKind::Graveyard)
        .sorcery_speed()
        .targets(&[targets::CREATURE])],
    ..oracle::QARSI_REVENANT
};
