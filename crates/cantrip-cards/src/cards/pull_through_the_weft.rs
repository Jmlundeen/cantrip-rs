use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// The targets of the spec at `spec` that are still legal (CR 608.2b), in the order chosen (CR 601.2c).
fn legal_for(g: &dyn Game, item: &StackItem, spec: usize) -> Vec<ObjectId> {
    g.state().spec_targets(item, spec).filter_map(|slot| targets::object(g, item, slot)).collect()
}

/// "Return up to two target nonland permanent cards from your graveyard to your hand, then return up to two target land cards
/// from your graveyard to the battlefield tapped" (CR 608.2c, 400.3): the nonland cards come back together, and then, as a
/// second step, the lands enter together tapped (CR 110.5b). Each group is whatever of its targets is still legal (CR 608.2b).
fn return_nonland_cards_then_lands(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards = legal_for(g, item, 0);
    g.return_to_hand(&cards);
    let lands: Vec<(ObjectId, PlayerId, bool)> = legal_for(g, item, 1).into_iter().map(|land| (land, item.controller, true)).collect();
    g.put_all_onto_battlefield_as(&lands)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::NONLAND_PERMANENT_CARD_IN_YOUR_GRAVEYARD.between(0, 2), targets::LAND_CARD_IN_YOUR_GRAVEYARD.between(0, 2)],
        return_nonland_cards_then_lands,
    ),
    ..oracle::PULL_THROUGH_THE_WEFT
};
