use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Target creature you control gains shroud until end of turn.
fn gain_shroud(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::Shroud])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(Cost::sacrifice(1, targets::filters::land), gain_shroud, "Sacrifice a land: Target creature you control gains shroud until end of turn.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::SYLVAN_SAFEKEEPER
};
