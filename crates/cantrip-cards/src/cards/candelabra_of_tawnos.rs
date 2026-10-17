use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // X target lands: the slot is counted by the X announced, and asked once it is (CR 601.2b-c).
    abilities: &[ability(cost("{X}, {T}"), effects::untap_targets, "{X}, {T}: Untap X target lands.")
        .targets(&[targets::LAND.counted()])],
    ..oracle::CANDELABRA_OF_TAWNOS
};
