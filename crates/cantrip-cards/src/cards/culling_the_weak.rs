use cantrip_api::*;
use cantrip_oracle as oracle;

/// Add {B}{B}{B}{B}.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(item.controller, Mana::B, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    // As an additional cost to cast this spell, sacrifice a creature (CR 118.8).
    additional_cost: Some(Cost::sacrifice(1, cantrip_mechanics::targets::filters::creature)),
    ..oracle::CULLING_THE_WEAK
};
