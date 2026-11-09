use cantrip_api::*;
use cantrip_mechanics::{effects, exert};
use cantrip_oracle as oracle;

/// "If this creature hasn't been exerted this turn" restricts the optional attack cost (CR 508.1g,
/// 701.43d), not the linked trigger. New objects have different identities (CR 400.7).
fn not_exerted_this_turn(q: &dyn Query, me: &Asker) -> bool {
    !q.history().this_turn().iter().any(|record| matches!(record.event, Event::Exerted { permanent, .. } if permanent == me.source))
}

/// "When you do, untap all other creatures you control and after this phase, there is an additional
/// combat phase" (CR 607.2h, 701.26b, 500.8). Both instructions happen on this trigger's resolution.
fn untap_and_extra_combat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::untap_other_creatures_you_control(g, item)?;
    g.add_phases(PhaseAnchor::Current, &[PhaseSpec::Whole(Phase::Combat)]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[exert::as_it_attacks(
        untap_and_extra_combat,
        "If this creature hasn't been exerted this turn, you may exert it as it attacks. When you do, untap all other creatures you control and after this phase, there is an additional combat phase.",
    ).attack_condition(not_exerted_this_turn)],
    ..oracle::COMBAT_CELEBRANT
};
