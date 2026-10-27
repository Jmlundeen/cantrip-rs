use cantrip_api::*;
use cantrip_mechanics::{damage_target, effects, targets};
use cantrip_oracle as oracle;

/// Mode 3: destroy target artifact.
fn destroy_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// Mode 4: Kolaghan's Command deals 2 damage to any target.
fn damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::return_first_target_to_hand, "Return target creature card from your graveyard to your hand.")
        .targets(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD]),
    mode(effects::target_player_discards::<1>, "Target player discards a card.")
        .targets(&[targets::PLAYER]),
    mode(destroy_artifact, "Destroy target artifact.")
        .targets(&[targets::ARTIFACT]),
    mode(damage, "Kolaghan's Command deals 2 damage to any target.")
        .targets(&[targets::ANY]),
])
    .choose(2, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::KOLAGHANS_COMMAND };
