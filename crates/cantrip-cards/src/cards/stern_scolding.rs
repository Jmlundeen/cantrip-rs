use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// "Target creature spell with power or toughness 2 or less" (CR 115.1): a spell on the stack, read as it is there.
fn small_creature_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::SPELL.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if {
            let c = q.chars(id);
            c.is(CardType::Creature) && (c.power.is_some_and(|p| p <= 2) || c.toughness.is_some_and(|t| t <= 2))
        })
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: small_creature_spell }], effect }),
    ..oracle::STERN_SCOLDING
};
