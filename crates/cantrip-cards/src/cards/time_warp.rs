use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target player takes an extra turn after this one" (CR 500.7). A target that is gone does nothing (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Some(player) = item.targets[0].player()
    {
        g.add_extra_turns(player, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::PLAYER], effect), ..oracle::TIME_WARP };
