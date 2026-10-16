use cantrip_api::*;
use cantrip_mechanics::{gift, pump, targets, tokens};
use cantrip_oracle as oracle;

/// "Target creature you control gets +2/+2 until end of turn. If the gift was promised, that creature also gains
/// indestructible until end of turn." The gift itself was given before this runs (CR 702.174j); a spell whose target has
/// become illegal does nothing (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 2, 2);
    if item.gift.is_some() {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Indestructible])), Duration::EndOfTurn);
    }
    Ok(())
}

/// "Gift a Food" (CR 702.174d): the Food is a token of its own.
pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    gift: Some(gift::FOOD),
    ..oracle::CRUMB_AND_GET_IT
};
