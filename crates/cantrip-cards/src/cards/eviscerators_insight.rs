use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect }),
    // As an additional cost to cast this spell, sacrifice an artifact or creature (CR 118.8); flashback pays it too.
    additional_cost: Some(Cost::sacrifice(1, cantrip_mechanics::targets::filters::artifact_or_creature)),
    ways: &[flashback::way(mana(4, 0, 0, 1, 0, 0), "Flashback {4}{B}")],
    ..oracle::EVISCERATORS_INSIGHT
};
