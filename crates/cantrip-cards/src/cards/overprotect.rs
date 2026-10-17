use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature you control gets +3/+3 and gains trample, hexproof, and indestructible until end of turn": all of it
/// from this spell's resolution (CR 611.2a), and none of it if the target is no longer legal (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        cantrip_mechanics::pump(g, item.object, creature, 3, 3);
        g.add_effect(
            item.object,
            creature,
            EffectKind::AddKeywords(Keywords::of(&[Keyword::Trample, Keyword::Hexproof, Keyword::Indestructible])),
            Duration::EndOfTurn,
        );
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE_YOU_CONTROL], effect), ..oracle::OVERPROTECT };
