use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::destroy_target(g, item, 0);
    Ok(())
}

/// "You and target opponent each draw a card": you first, then the opponent if still a legal target.
fn draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    if g.target_legal(item, 0)
        && let Some(player) = item.targets[0].player()
    {
        g.draw(player, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, destroy, "When Loran enters, destroy up to one target artifact or enchantment.")
            .targets(&[TargetSpec { min: 0, max: 1, ..targets::ARTIFACT_OR_ENCHANTMENT }]),
        ability(TAP, draw, "{T}: You and target opponent each draw a card.")
            .targets(&[targets::OPPONENT]),
    ],
    ..oracle::LORAN_OF_THE_THIRD_PATH
};
