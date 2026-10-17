use cantrip_api::*;
use cantrip_mechanics::{paradigm, targets};
use cantrip_oracle as oracle;

/// "Target player draws two cards and loses 2 life." Both happen to the player the spell targeted, if it is still a legal
/// target (CR 608.2b); the life is lost even when the library was empty (CR 121.4, 119.3).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        g.draw(player, 2);
        g.lose_life(player, 2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    paradigm: Some(paradigm::SPELL_ABILITY),
    delayed: &[paradigm::TRIGGER],
    ..oracle::DECORUM_DISSERTATION
};
