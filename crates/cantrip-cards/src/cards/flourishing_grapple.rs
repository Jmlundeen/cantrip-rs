use cantrip_api::*;
use cantrip_mechanics::{fight, targets};
use cantrip_oracle as oracle;

/// "Target creature or planeswalker an opponent controls that's red or white" (CR 115.1, 105.2): the colors as it is now,
/// the color words read through this spell's words (CR 612.2), and checked again as it resolves (CR 608.2b).
fn red_or_white_opposing(q: &dyn Query, me: &Asker, t: Target) -> bool {
    let words = q.words(me.source);
    (targets::CREATURE_OR_PLANESWALKER_AN_OPPONENT_CONTROLS.filter)(q, me, t)
        && matches!(t, Target::Object(id) if {
            let colors = q.chars(id).colors;
            colors.has(words.color(Color::Red)) || colors.has(words.color(Color::White))
        })
}

/// "... loses all abilities until end of turn. Target creature you control deals damage equal to its power to that
/// permanent" (CR 613.1f, 120.3): each target judged on its own (CR 608.2b) - with the first gone nothing is dealt
/// damage, and with the second gone the first still loses its abilities.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let victim = targets::object(g, item, 0);
    if let Some(id) = victim {
        g.add_effect(item.object, id, EffectKind::LoseAllAbilities, Duration::EndOfTurn);
    }
    if let (Some(creature), Some(victim)) = (targets::object(g, item, 1), victim) {
        fight::bite(g, creature, victim);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(red_or_white_opposing), targets::CREATURE_YOU_CONTROL], effect),
    ..oracle::FLOURISHING_GRAPPLE
};
