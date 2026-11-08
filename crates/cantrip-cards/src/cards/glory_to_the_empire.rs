use cantrip_api::*;
use cantrip_mechanics::{melee, statics, tokens, triggers};
use cantrip_oracle as oracle;

/// "create a number of 2/1 red Klingon Warrior creature tokens with haste equal to the number of opponents you have" (CR 111.1,
/// 608.2c): one token for each other player still in the game as the ability resolves, made from the shared token.
fn create_tokens(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let opponents = g.state().opponents(item.controller).count() as u32;
    tokens::create(g, item.controller, &tokens::KLINGON, opponents)?;
    Ok(())
}

/// "Creatures you control have melee." (CR 113.6, 613.1f): the shared melee trigger, granted as this card's delayed ability 0 to
/// each creature its controller controls, the enchantment as its source (CR 113.7).
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        create_tokens,
        "When this enchantment enters, create a number of 2/1 red Klingon Warrior creature tokens with haste equal to the number of opponents you have.",
    )],
    delayed: &[melee::TRIGGER],
    statics: &[static_ability(Layer::Ability, statics::your_creatures, statics::grant_delayed_ability, "Creatures you control have melee.")],
    ..oracle::GLORY_TO_THE_EMPIRE
};

pub static EXTRA: &[&CardDef] = &[&tokens::KLINGON];
