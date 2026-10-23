use cantrip_api::*;
use cantrip_mechanics::exile_all_until_leaves;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

/// "Exile each nonland permanent with mana value 2 or less until this enchantment leaves the battlefield" (CR 610.3):
/// every one, whoever controls it, exiled together as the ability resolves; a token's mana value is 0 (CR 202.3).
fn lock_down(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_all_until_leaves(g, item, |g, id| !g.chars(id).is(CardType::Land) && g.mana_value(id) <= 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, lock_down, "When this enchantment enters, exile each nonland permanent with mana value 2 or less until this enchantment leaves the battlefield.")],
    ..oracle::TEMPORARY_LOCKDOWN
};
