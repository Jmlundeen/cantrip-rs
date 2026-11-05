use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "When this creature enters, target player mills two cards. You gain 2 life." (CR 701.17a, 608.2b): with its one target
/// illegal as it resolves the whole ability does nothing, so the life is gained only when the player is still there to mill.
fn mill_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::target_player_mills::<2>(g, item)?;
    effects::gain_life(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::THIS_ENTERS, mill_and_gain, "When this creature enters, target player mills two cards. You gain 2 life.")
        .targets(&[targets::PLAYER])],
    ..oracle::WATCHER_OF_THE_WAYSIDE
};
