use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "Iroh's Demonstration deals 1 damage to each creature your opponents control": every such creature as it resolves,
/// all at once (CR 120.2).
fn opponents_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let hits: Vec<(ObjectId, Target, u16)> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.controller(id) != item.controller)
        .map(|id| (item.object, Target::Object(id), 1))
        .collect();
    g.damage_together(&hits);
    Ok(())
}

/// "Iroh's Demonstration deals 4 damage to target creature."
fn four_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 4);
    Ok(())
}

/// Choose one (CR 700.2).
static MODAL: ModalDef = modes(&[
    mode(opponents_creatures, "Iroh's Demonstration deals 1 damage to each creature your opponents control."),
    mode(four_damage, "Iroh's Demonstration deals 4 damage to target creature.").targets(&[targets::CREATURE]),
])
.choose(1, 1);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::IROHS_DEMONSTRATION };
