use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Destroy each artifact with mana value X or less." Every such artifact goes at once, in battlefield order.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = item.x as u32;
    let doomed: Vec<ObjectId> =
        g.state().battlefield().iter().copied().filter(|&id| g.chars(id).is(CardType::Artifact) && g.mana_value(id) <= x).collect();
    g.destroy_all(&doomed);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    ..oracle::MELTDOWN
};
