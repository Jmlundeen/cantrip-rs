use cantrip_api::*;
use cantrip_mechanics::{lands, targets};
use cantrip_oracle as oracle;

/// "Target Spacecraft" (CR 115.1, 205.3g): a permanent with the Spacecraft subtype, whether or not it is a creature right
/// now.
fn spacecraft(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.is_on_battlefield(id) && q.chars(id).has_subtype(Subtype::Spacecraft))
}

/// "Put two charge counters on target Spacecraft." (CR 122.6): the counters go on the Spacecraft's permanent, which is one
/// whether or not it is a creature right now (CR 205.3g, 305.7).
fn two_charge_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spacecraft) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(spacecraft, CounterKind::Charge, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        lands::ENTERS_MAY_FETCH_BASIC_TO_HAND,
        ability(
            cost("{1}, {T}"),
            two_charge_counters,
            "{1}, {T}: Put two charge counters on target Spacecraft. Activate only as a sorcery.",
        )
        .targets(&[target(spacecraft)])
        .sorcery_speed(),
    ],
    ..oracle::WARP_CORE
};
