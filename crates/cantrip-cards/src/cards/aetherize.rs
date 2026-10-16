use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Return all attacking creatures to their owner's hand." Every creature that is attacking as the spell resolves, whoever
/// controls it (CR 506.4), goes in one simultaneous move (CR 608.2f); a token returned ceases to exist (CR 111.7).
fn effect(g: &mut dyn Game, _item: &StackItem) -> Result<(), Suspend> {
    let attackers: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.is_attacking(id) && g.chars(id).is(CardType::Creature))
        .collect();
    g.return_to_hand(&attackers);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::AETHERIZE };
