use cantrip_api::*;
use cantrip_oracle as oracle;

/// Flip a coin; if you lose the flip, this artifact deals 3 damage to you. The flip is a two-sided die (CR 705.1) and
/// winning is its first face, as the C# engine reads it. The damage comes from the artifact even when it has left since
/// the trigger went on the stack (CR 113.7a, 608.2h).
fn flip(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.flip_coin(item.controller) {
        return Ok(());
    }
    g.damage(item.object, Target::Player(item.controller), 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_n(Mana::C, 2), "{T}: Add {C}{C}."),
        triggered(cantrip_mechanics::triggers::YOUR_UPKEEP, flip, "At the beginning of your upkeep, flip a coin. If you lose the flip, Mana Crypt deals 3 damage to you."),
    ],
    ..oracle::MANA_CRYPT
};
