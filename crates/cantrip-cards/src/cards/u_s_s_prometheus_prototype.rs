use cantrip_api::*;
use cantrip_mechanics::{station, statics, triggers};
use cantrip_oracle as oracle;

/// The 4+ striation, "Flying" with the 6/6 station box (CR 721.2b).
const FOUR_PLUS: &str = "4+ | Flying";

/// The 8+ striation's ability, Multivector Assault Mode (CR 721.2a).
const MULTIVECTOR: &str =
    "Whenever U.S.S. Prometheus deals combat damage to an opponent, it deals that much damage to each other opponent.";

const EIGHT_PLUS: &str = "8+ | Multivector Assault Mode";

/// "It deals that much damage to each other opponent" (CR 120.3, 608.2h, 101.4): the opponent hit is the trigger's subject and
/// takes nothing more; every other opponent is dealt the same amount at once.
fn deals_that_much_to_each_other_opponent(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(hit) = item.subject else { return Ok(()) };
    let amount = item.matched_damage_amount();
    let hits: Vec<(ObjectId, Target, u32)> = g
        .state()
        .opponents(item.controller)
        .filter(|&opponent| opponent != hit)
        .map(|opponent| (item.object, Target::Player(opponent), amount))
        .collect();
    g.damage_together(&hits);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[station::ability(
        "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 4+.)",
    )],
    statics: &[
        static_ability(Layer::Type, station::charged::<4>, station::is_a_creature, FOUR_PLUS),
        static_ability(Layer::Ability, station::charged::<4>, statics::have_flying, FOUR_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<4>, statics::base_pt::<6, 6>, FOUR_PLUS).continues(),
        static_ability(Layer::Ability, station::charged::<8>, station::has::<0>, EIGHT_PLUS),
    ],
    delayed: &[triggered(
        when(EventMask::DAMAGE, triggers::deals_combat_damage_to_an_opponent),
        deals_that_much_to_each_other_opponent,
        MULTIVECTOR,
    )
    .damage_amount()],
    ..oracle::U_S_S_PROMETHEUS_PROTOTYPE
};
