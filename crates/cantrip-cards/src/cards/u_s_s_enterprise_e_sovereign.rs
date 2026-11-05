use cantrip_api::*;
use cantrip_mechanics::{station, statics, triggers};
use cantrip_oracle as oracle;

const EIGHT_PLUS: &str = "8+ | Flying, vigilance";

/// "Destroy all tapped creatures" (CR 701.8): every tapped creature on the battlefield, both players', destroyed at
/// once. A creature that is untapped as the ability resolves is not one of them.
fn destroy_all_tapped_creatures(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let tapped: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.obj(id).tapped)
        .collect();
    instruction(g, |g| {
        g.destroy_all(&tapped);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, destroy_all_tapped_creatures, "When U.S.S. Enterprise-E enters, destroy all tapped creatures."),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, station::charged::<8>, statics::have_flying_and_vigilance, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<8>, statics::base_pt::<7, 7>, EIGHT_PLUS).continues(),
    ],
    ..oracle::U_S_S_ENTERPRISE_E_SOVEREIGN
};
