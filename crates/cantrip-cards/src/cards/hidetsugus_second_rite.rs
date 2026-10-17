use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "If target player has exactly 10 life, Hidetsugu's Second Rite deals 10 damage to that player." The life total is
/// looked at as the spell resolves, never when it is cast (CR 608.2c): a target that is no longer legal does nothing
/// (CR 608.2b), and a life total of anything but 10 is a spell that does nothing and finishes resolving.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0)
        && g.state().player(player).life == 10
    {
        g.damage(item.object, Target::Player(player), 10);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::PLAYER], effect), ..oracle::HIDETSUGUS_SECOND_RITE };
