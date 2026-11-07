use cantrip_api::*;
use cantrip_mechanics::goad;
use cantrip_oracle as oracle;

/// "Goad all creatures you don't control." (CR 701.15a): every creature on the battlefield controlled by another
/// player is goaded by this spell's controller until that player's next turn. The caster's own creatures are not.
fn goad_all(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.controller(id) != item.controller)
        .collect();
    g.one_instruction(&mut |g| {
        for &creature in &creatures {
            goad::goad(g, item, creature);
        }
        Ok(())
    })
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], goad_all),
    ..oracle::DISRUPT_DECORUM
};
