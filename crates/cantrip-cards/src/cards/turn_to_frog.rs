use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Until end of turn, target creature loses all abilities and becomes a blue Frog with base power and toughness 1/1."
/// Continuous effects from a resolved spell (CR 611.2), each in its layer and all at this spell's timestamp (CR 613.7a):
/// layer 4, its creature types are replaced by Frog and its card types and supertypes stay (CR 205.1a, the ruling: an
/// artifact creature stays one); layer 5, blue and no other color (CR 105.3); layer 6, it loses every ability it has by then,
/// and keeps one gained later (CR 613.7, the ruling); layer 7b, base 1/1, over every earlier base-setting effect and under a
/// later one (the ruling), and under whatever modifies power and toughness, counters included (layer 7c: a Giant Growth
/// before it still makes a 4/4). Abilities that have already triggered or been activated still resolve (the ruling). It
/// lasts until the cleanup step (CR 514.2). Frog and blue are words of its text (CR 612). With the target gone or no longer
/// a creature, nothing happens (CR 608.2b).
fn frog(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let words = g.words(item.object);
    let source = item.object;
    g.add_effect(source, creature, EffectKind::LoseAllAbilities, Duration::EndOfTurn);
    g.add_effect(source, creature, EffectKind::SetCreatureType(words.subtype(Subtype::Frog)), Duration::EndOfTurn);
    g.add_effect(source, creature, EffectKind::SetColors(words.colors(ColorSet::of(Color::Blue))), Duration::EndOfTurn);
    g.add_effect(source, creature, EffectKind::SetPt(1, 1), Duration::EndOfTurn);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], frog), ..oracle::TURN_TO_FROG };
