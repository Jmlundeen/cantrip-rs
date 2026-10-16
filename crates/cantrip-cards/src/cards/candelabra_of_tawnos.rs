use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Untap X target lands: every one that is still a legal target.
fn untap_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for land in targets::legal_objects(g, item) {
        g.untap(land);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // X target lands: the slot is counted by the X announced, and asked once it is (CR 601.2b-c).
    abilities: &[ability(cost("{X}, {T}"), untap_lands, "{X}, {T}: Untap X target lands.")
        .targets(&[targets::LAND.counted()])],
    ..oracle::CANDELABRA_OF_TAWNOS
};
