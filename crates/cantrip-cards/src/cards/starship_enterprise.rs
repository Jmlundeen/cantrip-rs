use cantrip_api::*;
use cantrip_mechanics::{effects, station, statics, triggers};
use cantrip_oracle as oracle;

/// The 16+ striation (CR 721.2a-b): at sixteen charge counters it is a 4/6 artifact creature with flying, the "artifact
/// creature at 16+" of its station reminder text.
const SIXTEEN_PLUS: &str = "16+ | Flying";

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::LANDFALL,
            effects::charge_counters_on_self::<4>,
            "Landfall \u{2014} Whenever a land you control enters, put four charge counters on Starship Enterprise.",
        ),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 16+.)"),
        // "Draw that many cards": as many as the combat damage it dealt to the player (CR 121.1, 603.2c).
        triggered(
            triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_EVENT,
            effects::draw_that_many,
            "Whenever Starship Enterprise deals combat damage to a player, draw that many cards.",
        )
        .damage_amount(),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<16>, station::is_a_creature, SIXTEEN_PLUS),
        static_ability(Layer::Ability, statics::itself, statics::have_flying, SIXTEEN_PLUS).continues(),
        static_ability(Layer::PtSet, statics::itself, statics::base_pt::<4, 6>, SIXTEEN_PLUS).continues(),
    ],
    ..oracle::STARSHIP_ENTERPRISE
};
