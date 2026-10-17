use cantrip_api::*;
use cantrip_mechanics::{delirium, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +1/+0 and gains first strike until end of turn. Delirium - If there are four or more card types among
/// cards in your graveyard, that creature gains double strike until end of turn" (CR 611.2a). Nothing if the target is no
/// longer legal (CR 608.2b). Delirium is counted as the spell resolves, when the spell itself is still on the stack.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, creature, EffectKind::Pump(1, 0), Duration::EndOfTurn);
    let mut keywords = Keywords::of(&[Keyword::FirstStrike]);
    if delirium(g, item.controller) {
        keywords = keywords.with(Keyword::DoubleStrike);
    }
    g.add_effect(item.object, creature, EffectKind::AddKeywords(keywords), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::VIOLENT_URGE };
