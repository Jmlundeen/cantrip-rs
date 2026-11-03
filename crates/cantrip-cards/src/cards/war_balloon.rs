use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::statics::creature_too;
use cantrip_oracle as oracle;

/// "{1}: Put a fire counter on this Vehicle." The counter goes only on this Vehicle while it is still on the battlefield
/// (CR 400.7).
fn fire_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Fire, 1);
    Ok(())
}

/// "As long as this Vehicle has three or more fire counters on it" (CR 122.1, 613.1d): only fire counters count.
fn has_three_or_more_fire_counters(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && q.state().obj(source).counters(CounterKind::Fire) >= 3
}

/// Flying is a core keyword read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}"), fire_counter, "{1}: Put a fire counter on this Vehicle."),
        crew(3, "Crew 3 (Tap any number of creatures you control with total power 3 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    statics: &[static_ability(
        Layer::Type,
        has_three_or_more_fire_counters,
        creature_too,
        "As long as this Vehicle has three or more fire counters on it, it's an artifact creature.",
    )],
    ..oracle::WAR_BALLOON
};
