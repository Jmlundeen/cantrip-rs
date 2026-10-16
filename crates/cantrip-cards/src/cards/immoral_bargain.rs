use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Destroy X target nonland permanents: every one that is still a legal target, together.
fn destroy_each(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.destroy_all(&targets::legal_objects(g, item));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "As an additional cost to cast this spell, sacrifice X creatures": X is announced though the mana cost has no {X}
    // (CR 601.2b), and the targets are counted by it.
    additional_cost: Some(Cost { variable: Some(VariableCost::SacrificeX(targets::filters::creature)), ..Cost::NONE }),
    spell: spell(&[targets::NONLAND_PERMANENT.counted()], destroy_each),
    ..oracle::IMMORAL_BARGAIN
};
