use cantrip_api::*;
use cantrip_mechanics::{damage_target, destroy_target, effects, targets, tokens};
use cantrip_oracle as oracle;

/// "Prismari Command deals 2 damage to any target."
fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

/// "Target player draws two cards, then discards two cards."
fn loot_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::target_player_draws_then_discards(g, item, 0, 2)
}

/// "Target player creates a Treasure token."
fn treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::target_player_creates_treasures(g, item, 0, 1)
}

/// "Destroy target artifact."
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(damage, "Prismari Command deals 2 damage to any target.").targets(&[targets::ANY]),
    mode(loot_two, "Target player draws two cards, then discards two cards.").targets(&[targets::PLAYER]),
    mode(treasure, "Target player creates a Treasure token.").targets(&[targets::PLAYER]),
    mode(destroy_artifact, "Destroy target artifact.").targets(&[targets::ARTIFACT]),
])
.choose(2, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::PRISMARI_COMMAND };

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
