use cantrip_api::*;
use cantrip_mechanics::tokens::{self, create_map};
use cantrip_oracle as oracle;

/// Draw two cards and create a Map token.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    create_map(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // As an additional cost to cast this spell, sacrifice an artifact or creature (CR 118.8).
    additional_cost: Some(Cost::sacrifice(1, cantrip_mechanics::targets::filters::artifact_or_creature)),
    ..oracle::FANATICAL_OFFERING
};

pub static EXTRA: &[&CardDef] = &[&tokens::MAP];
