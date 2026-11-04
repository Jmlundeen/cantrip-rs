use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target Spacecraft or Planet you control" (CR 115.1, 205.3g, 205.3i): a permanent on the battlefield under the spell's
/// controller's control with the Spacecraft or the Planet subtype, judged again as the spell resolves (CR 608.2b).
fn spacecraft_or_planet_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_you_control_filter(q, asker, t)
        && matches!(t, Target::Object(id) if q.chars(id).has_subtype(Subtype::Spacecraft) || q.chars(id).has_subtype(Subtype::Planet))
}

/// "Put five charge counters on target Spacecraft or Planet you control."
fn five_charge_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(object) = targets::object(g, item, 0) {
        g.add_counters(object, CounterKind::Charge, 5);
    }
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(five_charge_counters, "Put five charge counters on target Spacecraft or Planet you control.")
        .targets(&[target(spacecraft_or_planet_you_control)]),
    mode(effects::destroy_first_target, "Destroy target artifact.").targets(&[targets::ARTIFACT]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::DRILL_TOO_DEEP };
