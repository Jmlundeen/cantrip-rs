use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Counter target spell.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect }),
    // As an additional cost to cast this spell, return a creature you control to its owner's hand (CR 118.8).
    additional_cost: Some(Cost::return_to_hand(1, targets::filters::creature)),
    ..oracle::FAMILIARS_RUSE
};
