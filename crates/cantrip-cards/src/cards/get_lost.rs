use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::tokens::MAP;
use cantrip_oracle as oracle;

/// "Target creature, enchantment, or planeswalker."
fn creature_enchantment_or_planeswalker(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) || c.is(CardType::Enchantment) || c.is(CardType::Planeswalker)
    })
}

/// Destroy target creature, enchantment, or planeswalker. Its controller (the one it last had, CR 608.2h) creates two
/// Map tokens, together, whether or not it was destroyed.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = cantrip_mechanics::targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(permanent);
    g.destroy(permanent);
    cantrip_mechanics::tokens::create(g, controller, &MAP, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef {
        targets: &[TargetSpec { min: 1, max: 1, filter: creature_enchantment_or_planeswalker }],
        effect,
    }),
    ..oracle::GET_LOST
};

pub static EXTRA: &[&CardDef] = &[&MAP];
