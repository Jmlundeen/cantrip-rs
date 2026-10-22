use cantrip_api::*;
use cantrip_mechanics::manifest_dread::manifest_dread;
use cantrip_mechanics::{crew::crew, effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Creature with toughness 5 or less."
fn target_filter(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.chars(id).toughness.is_some_and(|v| v <= 5))
}

/// "When this Vehicle leaves the battlefield, the exiled card's owner manifests dread." The cards exiled with this
/// Vehicle (CR 607.2a): normally one, but if its enter ability triggered more than once, each player who owns one or more
/// of them manifests dread, once, in turn order from the active player (its ruling; CR 101.4). With nothing exiled (the
/// enter ability has not resolved, or exiled nothing) it does nothing.
fn owner_manifests_dread(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let owners: Vec<PlayerId> = g.exiled_with(item.object).into_iter().map(|card| g.obj(card).owner).collect();
    let players: Vec<PlayerId> = g.state().apnap().filter(|p| owners.contains(p)).collect();
    for player in players {
        if !cantrip_mechanics::players::reached(g, player) {
            continue;
        }
        manifest_dread(g, player, item.object)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, effects::exile_first_target_with_this, "When this Vehicle enters, exile up to one target creature with toughness 5 or less.")
            .targets(&[up_to_one(target_filter)]),
        triggered(triggers::THIS_LEAVES, owner_manifests_dread, "When this Vehicle leaves the battlefield, the exiled card's owner manifests dread."),
        crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::UNIDENTIFIED_HOVERSHIP
};
