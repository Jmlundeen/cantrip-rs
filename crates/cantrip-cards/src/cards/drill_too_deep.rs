use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target Spacecraft or Planet you control" (CR 115.1, 205.3g, 205.3i): a permanent on the battlefield under the spell's
/// controller's control with the Spacecraft or the Planet subtype, judged again as the spell resolves (CR 608.2b).
fn spacecraft_or_planet_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_you_control_filter(q, asker, t)
        && matches!(t, Target::Object(id) if q.chars(id).has_subtype(Subtype::Spacecraft) || q.chars(id).has_subtype(Subtype::Planet))
}

static MODAL: ModalDef = modes(&[
    mode(effects::charge_counters_on_target::<5>, "Put five charge counters on target Spacecraft or Planet you control.")
        .targets(&[target(spacecraft_or_planet_you_control)]),
    mode(effects::destroy_first_target, "Destroy target artifact.").targets(&[targets::ARTIFACT]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::DRILL_TOO_DEEP };
