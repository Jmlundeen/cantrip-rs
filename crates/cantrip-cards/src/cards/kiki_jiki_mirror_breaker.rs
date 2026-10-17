use cantrip_api::*;
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
/// beginning of the next end step" (`copy::create_hasty_token_copy_sacrificed_at_end_step`, CR 707.2, 707.9a, 603.7). Its
/// "enters" abilities trigger and its "enters" replacements apply (the ruling).
fn copy_with_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let me = g.card_id(&CARD);
    cantrip_mechanics::copy::create_hasty_token_copy_sacrificed_at_end_step(g, item, me, 0)
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, copy_with_haste, "{T}: Create a token that's a copy of target nonlegendary creature you control, except it has haste. Sacrifice it at the beginning of the next end step.")
        .targets(&[target(nonlegendary_creature_you_control)])],
    delayed: &[cantrip_mechanics::copy::SACRIFICE_IT_AT_END_STEP],
    ..oracle::KIKI_JIKI_MIRROR_BREAKER
};
