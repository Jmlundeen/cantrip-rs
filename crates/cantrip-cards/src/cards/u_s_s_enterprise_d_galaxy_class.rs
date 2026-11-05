use cantrip_api::*;
use cantrip_mechanics::{library, station, statics};
use cantrip_oracle as oracle;

const SEVEN_PLUS: &str = "7+ | Flying, vigilance";

/// "Whenever one or more charge counters are put on U.S.S. Enterprise-D for the first time each turn" (CR 122.6, 603.2c):
/// one event however many counters it put, and a trigger condition asked of the history as it was just before this event
/// (CR 603.2 - a trigger condition, not a limit on how often the ability triggers, so an effect that makes it trigger
/// additional times still does, CR 603.2d): no charge counters went on this object earlier this turn, whether or not it
/// had this ability then.
fn first_charge_counters_this_turn(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::CountersPut { object, kind: CounterKind::Charge, .. }
            if *object == me.source
                && !q.history().this_turn().iter().any(|r| {
                    matches!(r.event, Event::CountersPut { object, kind: CounterKind::Charge, .. } if object == me.source)
                }) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        // The exiled card may be played this turn by its controller.
        triggered(
            when_asked(EventMask::COUNTERS, first_charge_counters_this_turn),
            library::exile_top_card_play_this_turn,
            "Whenever one or more charge counters are put on U.S.S. Enterprise-D for the first time each turn, exile the top card of your library. You may play that card this turn.",
        ),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 7+.)"),
    ],
    // The 7+ striation (CR 721.2a-b): at seven or more charge counters it is an artifact creature 4/5 with flying and
    // vigilance.
    statics: &[
        static_ability(Layer::Type, station::charged::<7>, station::is_a_creature, SEVEN_PLUS),
        static_ability(Layer::Ability, station::charged::<7>, statics::have_flying_and_vigilance, SEVEN_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<7>, statics::base_pt::<4, 5>, SEVEN_PLUS).continues(),
    ],
    ..oracle::U_S_S_ENTERPRISE_D_GALAXY_CLASS
};
