use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

// Exile the legal targets together, then deal 5 damage to the controller (CR 608.2b, 701.13, 120.3).
fn resolve(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.exile_all(&targets::legal_objects(g, item));
    g.end_sentence();
    g.damage(item.object, Target::Player(item.controller), 5);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONARTIFACT_CREATURE.between(2, 2)], resolve),
    ..oracle::ASHES_TO_ASHES
};
