use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Until end of turn, target creature you control becomes a [creature type] with base power and toughness [p]/[t] and
/// gains [keywords]." The creature type is set, replacing every other creature type and nothing else (CR 205.1a); the base
/// power and toughness are set in layer 7b (CR 613.4b), so counters and pumps count on top. Read through the words of the
/// spell, which a text-changing effect may have changed (CR 612).
fn becomes(g: &mut dyn Game, item: &StackItem, kind: Subtype, power: i16, toughness: i16, keywords: Keywords) {
    let Some(creature) = targets::object(g, item, 0) else { return };
    let words = g.words(item.object);
    instruction(g, |g| {
        g.add_effect(item.object, creature, EffectKind::SetCreatureType(words.subtype(kind)), Duration::EndOfTurn);
        g.add_effect(item.object, creature, EffectKind::SetPt(power, toughness), Duration::EndOfTurn);
        g.add_effect(item.object, creature, EffectKind::AddKeywords(keywords), Duration::EndOfTurn);
    });
}

/// Conceal: a Citizen 1/1 with hexproof.
fn conceal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    becomes(g, item, Subtype::Citizen, 1, 1, Keywords::of(&[Keyword::Hexproof]));
    Ok(())
}

/// Reveal: a Hero 3/4 with flying and vigilance.
fn reveal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    becomes(g, item, Subtype::Hero, 3, 4, Keywords::of(&[Keyword::Flying, Keyword::Vigilance]));
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(conceal, "Conceal \u{2014} Until end of turn, target creature you control becomes a Citizen with base power and toughness 1/1 and gains hexproof.")
        .targets(&[targets::CREATURE_YOU_CONTROL]),
    mode(reveal, "Reveal \u{2014} Until end of turn, target creature you control becomes a Hero with base power and toughness 3/4 and gains flying and vigilance.")
        .targets(&[targets::CREATURE_YOU_CONTROL]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SECRET_IDENTITY };
