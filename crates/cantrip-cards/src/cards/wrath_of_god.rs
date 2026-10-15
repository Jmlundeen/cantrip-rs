use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Destroy all creatures. They can't be regenerated." Every creature goes at once, in battlefield order.
fn effect(g: &mut dyn Game, _: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    g.destroy_all_no_regen(&creatures);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ..oracle::WRATH_OF_GOD
};
