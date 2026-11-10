use cantrip_api::*;
use cantrip_mechanics::lands_you_control;
use cantrip_mechanics::statics::{attacks_each_combat_if_able, itself};
use cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_EVENT;
use cantrip_oracle as oracle;

const TRIGGER: &str = "Whenever this creature deals combat damage to a player, put a +1/+1 counter on it for each land that player controls.";

/// "Whenever this creature deals combat damage to a player, put a +1/+1 counter on it for each land that player controls"
/// (CR 510.2, 603.2c, 122.1): the trigger's subject is the damaged player, whose lands are counted as the ability resolves,
/// and the counters are put on it together, as one event.
fn counters_for_their_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(damaged) = item.subject else { return Ok(()) };
    let lands = lands_you_control(g, damaged) as i32;
    g.add_counters(item.object, CounterKind::PlusOne, lands);
    Ok(())
}

// Flying and indestructible are the core's, read from the card's JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(DEALS_COMBAT_DAMAGE_TO_PLAYER_EVENT, counters_for_their_lands, TRIGGER)],
    statics: &[static_ability(Layer::Ability, itself, attacks_each_combat_if_able, "This creature attacks each combat if able.")],
    ..oracle::PLANET_KILLER
};
