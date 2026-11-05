use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature you control gains deathtouch and indestructible until end of turn" (CR 702.2b, 702.12b, 611.2a): one layer 6
/// change for the rest of the turn, if the creature is still a legal target as the spell resolves (CR 608.2b).
fn gain_deathtouch_and_indestructible(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Deathtouch, Keyword::Indestructible])), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE_YOU_CONTROL], gain_deathtouch_and_indestructible), ..oracle::ALESHAS_LEGACY };
