use cantrip_api::*;
use cantrip_oracle as oracle;

/// \"Sulu stations Spacecraft and crews Vehicles as though his power were tripled\" (CR 702.122b, 702.184a): only the
/// station of a Spacecraft and the crew of a Vehicle, so a teammate's teamwork or a saddle cost still counts his 2, and a
/// Vehicle that is not a Spacecraft crewing its own crew cost sees him at 6. The rule is asked as the cost is paid, of the
/// power he has then (CR 107.1b: doubled negative power is more negative), which is why it takes that power in.
fn triples_for_spacecraft_and_vehicles(q: &dyn Query, _: &Asker, power: i16, permanent: ObjectId) -> i16 {
    let subtypes = q.chars(permanent).subtypes;
    if subtypes.has(Subtype::Vehicle) || subtypes.has(Subtype::Spacecraft) {
        power.saturating_mul(3)
    } else {
        power
    }
}

pub static CARD: CardDef = CardDef {
    tap_power: &[triples_for_spacecraft_and_vehicles],
    ..oracle::SULU_HEROIC_HELMSMAN
};
