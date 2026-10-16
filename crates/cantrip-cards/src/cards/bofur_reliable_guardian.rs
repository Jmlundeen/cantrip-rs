use cantrip_api::*;
use cantrip_mechanics::effects::gain_hexproof_and_indestructible_until_end_of_turn as protect;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target artifact or creature you control gains hexproof and indestructible until end of turn": nothing happens if the
/// target is no longer legal (CR 608.2b).
fn concerted_care(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(permanent) = targets::object(g, item, 0) {
        protect(g, item.object, permanent);
    }
    Ok(())
}

/// The Adventure (CR 715): Concerted Care.
pub static CONCERTED_CARE: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE_YOU_CONTROL], concerted_care),
    ..oracle::CONCERTED_CARE
};

pub static CARD: CardDef = CardDef { adventure: Some(&CONCERTED_CARE), ..oracle::BOFUR_RELIABLE_GUARDIAN };

pub static EXTRA: &[&CardDef] = &[&CONCERTED_CARE];
