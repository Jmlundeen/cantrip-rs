use cantrip_api::*;
use cantrip_mechanics::triggers::cycling;
use cantrip_oracle as oracle;

/// Return all artifacts to their owners' hands: the set is fixed before anything moves, and they move together
/// (CR 608.2c, 603.10a).
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let artifacts: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Artifact))
        .collect();
    g.return_to_hand(&artifacts);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    abilities: &[cycling(mana(2, 0, 0, 0, 0, 0), "Cycling {2} ({2}, Discard this card: Draw a card.)")],
    ..oracle::REBUILD
};
