use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// Target creature gains banding until end of turn (CR 611.2a, 702.22).
fn grant_banding(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Banding);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}").and(TAP), grant_banding, "{1}, {T}: Target creature gains banding until end of turn.")
        .targets(&[targets::CREATURE])],
    ..oracle::HELM_OF_CHATZUK
};
