use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{artifacts_you_control, damage_target, statics, station, targets};
use cantrip_oracle as oracle;

const SIX_PLUS: &str = "6+ | Flying";

/// "It deals damage equal to the number of artifacts you control to target creature an opponent controls" (CR 120.3, 608.2h): the
/// artifacts are counted as the ability resolves, the Gunship among them if it is still there, and the damage is the Gunship's (by last-known
/// information if it has left, CR 113.7a); nothing if the target is no longer legal (CR 608.2b).
fn blast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let artifacts = artifacts_you_control(g, item.controller);
    damage_target(g, item, 0, artifacts as u16);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, blast, "When this Spacecraft enters, it deals damage equal to the number of artifacts you control to target creature an opponent controls.")
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 6+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<6>, station::is_a_creature, SIX_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, SIX_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<4, 3>, SIX_PLUS).continues(),
    ],
    ..oracle::WARMAKER_GUNSHIP
};
