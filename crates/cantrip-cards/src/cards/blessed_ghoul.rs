use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "{2}{W/B}: Return this card from your graveyard to your hand." An activated ability that functions only in its owner's
/// graveyard (CR 113.6k), which `from` says.
fn return_from_graveyard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::return_this_to_hand(g, item)
}

/// Lifelink is a core keyword read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}{W/B}"), return_from_graveyard, "{2}{W/B}: Return this card from your graveyard to your hand.")
        .from(ZoneKind::Graveyard)],
    ..oracle::BLESSED_GHOUL
};
