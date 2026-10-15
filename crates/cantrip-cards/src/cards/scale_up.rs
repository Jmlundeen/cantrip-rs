use cantrip_api::*;
use cantrip_mechanics::{overload, targets};
use cantrip_oracle as oracle;

/// "Until end of turn, target creature you control becomes a green Wurm with base power and toughness 6/4." Overloaded
/// (CR 702.96a), "each creature you control" does: the creatures are those on the battlefield as it resolves (CR 611.2c).
/// "A green Wurm" sets the creature type, replacing every other creature type and nothing else (CR 205.1a), and the
/// color (CR 105.3); the base power and toughness are set in layer 7b (CR 613.4b), so counters and pumps count on top.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let creatures = overload::affected(g, item, |chars, controller| chars.is(CardType::Creature) && controller == you);
    for creature in creatures {
        g.add_effect(item.object, creature, EffectKind::SetCreatureType(Subtype::Wurm), Duration::EndOfTurn);
        g.add_effect(item.object, creature, EffectKind::SetColors(ColorSet::of(Color::Green)), Duration::EndOfTurn);
        g.add_effect(item.object, creature, EffectKind::SetPt(6, 4), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::CREATURE_YOU_CONTROL], effect }),
    overload: Some(mana(4, 0, 0, 0, 0, 2)),
    ..oracle::SCALE_UP
};
