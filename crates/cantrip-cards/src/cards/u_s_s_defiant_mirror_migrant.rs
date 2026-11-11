use cantrip_api::*;
use cantrip_mechanics::{station, statics, targets, triggers};
use cantrip_oracle as oracle;

/// The 8+ striation, "Flying" with the 5/5 box (CR 721.2b): while it has eight or more charge counters it has flying and is an
/// artifact creature with base power and toughness 5/5 in addition to its other types.
const EIGHT_PLUS: &str = "8+ | Flying (5/5)";

/// "It and up to one target creature you control phase out" (CR 702.26b): the Spacecraft and that creature together, and with
/// them, indirectly, whatever is attached to them (CR 702.26g). The target is judged as this resolves (CR 608.2b), so with no
/// target chosen only the Spacecraft phases out; both phase in during their controller's next untap step (CR 502.1, 702.26a).
fn phase_out_with_a_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut out = vec![item.object];
    out.extend(targets::object(g, item, 0));
    g.phase_out(&out);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            phase_out_with_a_creature,
            "When U.S.S. Defiant enters, it and up to one target creature you control phase out. (Treat them and anything attached to them as though they don't exist until your next turn.)",
        )
        .targets(&[targets::CREATURE_YOU_CONTROL.between(0, 1)]),
        station::ability("Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 8+.)"),
    ],
    statics: &[
        static_ability(Layer::Type, station::charged::<8>, station::is_a_creature, EIGHT_PLUS),
        static_ability(Layer::Ability, station::charged::<8>, statics::have_flying, EIGHT_PLUS).continues(),
        static_ability(Layer::PtSet, station::charged::<8>, statics::base_pt::<5, 5>, EIGHT_PLUS).continues(),
    ],
    ..oracle::U_S_S_DEFIANT_MIRROR_MIGRANT
};
