use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "When Abigale enters, up to one other target creature loses all abilities. Put a flying counter, a first strike counter,
/// and a lifelink counter on that creature." The loss lasts indefinitely (the ruling: not until cleanup, nor while Abigale
/// stays), and the counters are put on after it, so their keywords are kept (CR 613.7c; the ruling: an ability gained after
/// the ability resolves is kept). No target chosen, or one no longer legal (CR 608.2b), does nothing.
fn quiet(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, creature, EffectKind::LoseAllAbilities, Duration::Indefinite);
    g.add_counters(creature, CounterKind::Flying, 1);
    g.add_counters(creature, CounterKind::FirstStrike, 1);
    g.add_counters(creature, CounterKind::Lifelink, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, quiet, "When Abigale enters, up to one other target creature loses all abilities. Put a flying counter, a first strike counter, and a lifelink counter on that creature.")
        .targets(&[up_to_one(targets::other_creature_filter)])],
    ..oracle::ABIGALE_ELOQUENT_FIRST_YEAR
};
