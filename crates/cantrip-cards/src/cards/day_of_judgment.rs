use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Destroy all creatures." Every creature goes at once, in battlefield order (CR 701.8a, 101.4); regeneration shields
/// and indestructible are the verb's business, as it does not say they can't be regenerated.
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    g.destroy_all(&creatures);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ..oracle::DAY_OF_JUDGMENT
};
