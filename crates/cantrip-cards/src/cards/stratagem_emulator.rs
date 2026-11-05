use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

/// "When this creature dies, create X 1/1 colorless Borg artifact creature tokens, where X is its power." (CR 603.6c,
/// 608.2h): its power as it last existed on the battlefield, so a -1/-1 counter or a pump effect in response changes X.
fn tokens_equal_to_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = g.chars(item.object).power.unwrap_or(0).max(0) as u32;
    tokens::create(g, item.controller, &tokens::BORG, power)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_DIES,
        tokens_equal_to_power,
        "When this creature dies, create X 1/1 colorless Borg artifact creature tokens, where X is its power.",
    )],
    ..oracle::STRATAGEM_EMULATOR
};

pub static EXTRA: &[&CardDef] = &[&tokens::BORG];
