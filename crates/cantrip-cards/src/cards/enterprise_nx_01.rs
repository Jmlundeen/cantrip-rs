use cantrip_api::*;
use cantrip_mechanics::{statics, station, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Tap target creature and put a stun counter on it" (CR 701.26a, 122.1d). An already-tapped creature still gets the counter.
fn tap_and_stun(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.tap(creature);
        g.add_counters(creature, CounterKind::Stun, 1);
    }
    Ok(())
}

/// "6+ | Flying, vigilance" (CR 721.2a): both abilities are granted together while the striation is active.
fn flying_and_vigilance(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    statics::have_flying(q, source, chars);
    statics::have_vigilance(q, source, chars);
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            tap_and_stun,
            "When Enterprise NX-01 enters, tap target creature an opponent controls and put a stun counter on it.",
        )
        .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 6+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<6>, station::is_a_creature, "6+ | Flying, vigilance (4/3)"),
        static_ability(Layer::Ability, station::charged::<6>, flying_and_vigilance, "6+ | Flying, vigilance (4/3)").continues(),
        static_ability(Layer::PtSet, station::charged::<6>, statics::base_pt::<4, 3>, "6+ | Flying, vigilance (4/3)").continues(),
    ],
    ..oracle::ENTERPRISE_NX_01
};
