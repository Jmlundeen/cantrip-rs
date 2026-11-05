use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Put three +1/+1 counters on target creature you control" (CR 122.1a, 608.2b): none if the target is no longer legal as the
/// spell resolves, in which case it does not resolve at all.
fn three_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 3);
    Ok(())
}

/// The Omen (CR 720): "Put three +1/+1 counters on target creature you control. (Then shuffle this card into its owner's library.)"
/// Cast from the hand as an Omen sorcery for {2}{G}; as it resolves it is shuffled into its owner's library (CR 720.3d, the core's).
pub static DYNAMIC_SOAR: CardDef = CardDef { spell: spell(&[targets::CREATURE_YOU_CONTROL], three_counters), ..oracle::DYNAMIC_SOAR };

/// "You may cast sorcery spells and Dragon spells as though they had flash" (CR 307.5, 702.8): its controller's, from whatever zone
/// something lets them cast the card from, the spell judged as the face that would be cast - the Omen of a creature card is a
/// sorcery spell (CR 720.3a), the card itself a Dragon spell.
fn sorcery_or_dragon_you_cast(q: &dyn Query, asker: &Asker, spell: ObjectId, caster: PlayerId) -> bool {
    let chars = q.chars(spell);
    caster == asker.controller && (chars.is(CardType::Sorcery) || chars.has_subtype(q.words(asker.source).subtype(Subtype::Dragon)))
}

pub static CARD: CardDef = CardDef {
    flash_rules: &[sorcery_or_dragon_you_cast],
    omen: Some(&DYNAMIC_SOAR),
    ..oracle::WHIRLWING_STORMBROOD
};

pub static EXTRA: &[&CardDef] = &[&DYNAMIC_SOAR];
