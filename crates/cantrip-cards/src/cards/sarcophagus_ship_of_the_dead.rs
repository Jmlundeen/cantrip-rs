use cantrip_api::*;
use cantrip_mechanics::{statics, station, triggers};
use cantrip_oracle as oracle;

const STATION_TEXT: &str = "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 10+.)";
const TEN_PLUS: &str = "10+ | Flying";

/// "Whenever a creature dies, if this isn't a creature, put a charge counter on it." (CR 603.4, 121.1): the intervening
/// "if" reads its own card types through the layers, so a Spacecraft that is a creature when the trigger would trigger
/// neither triggers nor, if it became one before it resolves, puts a counter on.
fn charge_unless_a_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Charge, 1);
    Ok(())
}

fn not_a_creature(q: &dyn Query, asker: &Asker) -> bool {
    !q.chars(asker.source).is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::CREATURE_DIES, charge_unless_a_creature, "Whenever a creature dies, if this isn't a creature, put a charge counter on it.")
            .condition(not_a_creature),
        station::ability(STATION_TEXT),
    ],
    statics: &[
        static_ability(
            Layer::PtModify,
            statics::other_creatures_you_control,
            statics::plus_pt::<1, 0>,
            "Other creatures you control get +1/+0.",
        ),
        static_ability(Layer::Type, station::charged::<10>, station::is_a_creature, TEN_PLUS),
        static_ability(Layer::PtSet, station::charged::<10>, statics::base_pt::<4, 4>, TEN_PLUS).continues(),
        static_ability(Layer::Ability, station::charged::<10>, statics::have_flying, TEN_PLUS),
    ],
    ..oracle::SARCOPHAGUS_SHIP_OF_THE_DEAD
};
