use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// "You and target opponent each draw a card", in turn order from the active player (CR 121.2c).
fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::you_and_target_player_each_draw(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, destroy, "When Loran enters, destroy up to one target artifact or enchantment.")
            .targets(&[targets::ARTIFACT_OR_ENCHANTMENT.between(0, 1)]),
        ability(TAP, draw, "{T}: You and target opponent each draw a card.")
            .targets(&[targets::OPPONENT]),
    ],
    ..oracle::LORAN_OF_THE_THIRD_PATH
};
