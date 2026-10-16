use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature you control gets +1/+1 and gains hexproof until end of turn. Untap it." The target is checked as
/// the spell resolves (CR 608.2b); with it gone the spell does nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 1, 1);
    g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Hexproof])), Duration::EndOfTurn);
    g.untap(creature);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    ..oracle::OCTOPUS_FORM
};
