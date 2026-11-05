use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "When Saru enters, another target creature you control gains hexproof until end of turn" (CR 702.11b, 611.2a): a layer 6
/// keyword for the rest of the turn, granted to the target as the trigger resolves (CR 608.2b). Flash is a core keyword.
fn grant_hexproof(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        effects::gain_hexproof_until_end_of_turn(g, item.object, creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        grant_hexproof,
        "When Saru enters, another target creature you control gains hexproof until end of turn.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    ..oracle::SARU_UNTHREATENED
};
