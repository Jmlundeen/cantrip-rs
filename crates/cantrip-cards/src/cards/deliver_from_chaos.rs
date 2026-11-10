use cantrip_api::*;
use cantrip_mechanics::{targets, tokens};
use cantrip_oracle as oracle;

/// "Exile target artifact or creature. Create X 1/1 colorless Borg artifact creature tokens, where X is that permanent's mana
/// value." The mana value is the exiled permanent's own, read where it is still a permanent (CR 202.3, 608.2h) - the tokens are
/// made after the exile, when the card in exile is a new object (CR 400.7) and a token has ceased to exist - and X 0 makes no
/// tokens (CR 111.2). A
/// target that has left is not exiled and makes nothing (CR 608.2b).
fn deliver_from_chaos(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let x = g.mana_value(permanent);
    g.exile(permanent);
    g.end_sentence();
    if x > 0 {
        tokens::create_by_rule(g, item.controller, &tokens::BORG, x, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ARTIFACT_OR_CREATURE], deliver_from_chaos),
    ..oracle::DELIVER_FROM_CHAOS
};
