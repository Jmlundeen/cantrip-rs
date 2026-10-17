use cantrip_api::*;
use cantrip_mechanics::effects::{gain_hexproof_and_indestructible_until_end_of_turn as protect, gain_life};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target permanent you control gains hexproof and indestructible until end of turn. You gain 2 life." With its one target
/// illegal the spell doesn't resolve (CR 608.2b), so no life is gained; the keywords are a layer 6 effect for the rest of the
/// turn (CR 611.2a, 702.11b, 702.12b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        protect(g, item.object, permanent);
    }
    gain_life(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PERMANENT_YOU_CONTROL], effect),
    ..oracle::TAMIYOS_SAFEKEEPING
};
