use cantrip_api::*;
use cantrip_mechanics::delayed::{self, SACRIFICE_CREATED};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target nonlegendary creature you control" (CR 115.1, 205.4a), judged now: an animated land is a creature while it is
/// animated, and Kiki-Jiki itself is legendary.
fn nonlegendary_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.controller(id) == asker.controller && {
        let c = q.chars(id);
        c.is(CardType::Creature) && !c.supertypes.legendary
    })
}

/// "Create a token that's a copy of target nonlegendary creature you control, except it has haste. Sacrifice it at the
/// beginning of the next end step." The target's copiable values as the ability resolves (CR 707.2: what is printed on it, or
/// what it is copying, never counters, Auras or an animation; the rulings), with haste as an exception, so a copy of the
/// token has haste too (CR 707.9b) but is not sacrificed (the ruling). Its own "enters" abilities trigger and its "enters"
/// replacements apply (the ruling). The delayed ability is about every token this resolution made (CR 603.7c: a replacement
/// that makes more makes each of them sacrificed). Nothing if the target is no longer legal (CR 608.2b).
fn copy_with_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let values = g.copiable_values(creature).with_keyword(Keyword::Haste);
    let tokens = g.create_token_copies_of(values, item.controller, 1)?;
    delayed::at_next_end_step_about(g, item, 0, &tokens);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, copy_with_haste, "{T}: Create a token that's a copy of target nonlegendary creature you control, except it has haste. Sacrifice it at the beginning of the next end step.")
        .targets(&[target(nonlegendary_creature_you_control)])],
    delayed: &[SACRIFICE_CREATED],
    ..oracle::KIKI_JIKI_MIRROR_BREAKER
};
