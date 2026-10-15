use cantrip_api::*;
use cantrip_mechanics::{counter_target, counter_unless_pays};
use cantrip_oracle as oracle;

/// Counter target noncreature spell unless its controller pays {1}. Ferocious: if you control a creature with power 4
/// or greater, counter that spell instead. Ferocious is judged once, as the spell resolves (CR 608.2c): only the
/// upgraded effect happens, never both.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let ferocious = g
        .permanents_of(item.controller)
        .into_iter()
        .any(|id| g.chars(id).is(CardType::Creature) && g.chars(id).power.is_some_and(|p| p >= 4));
    if ferocious {
        counter_target(g, item, 0);
        return Ok(());
    }
    counter_unless_pays(g, item, 0, mana(1, 0, 0, 0, 0, 0))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[cantrip_mechanics::targets::NONCREATURE_SPELL], effect }),
    ..oracle::STUBBORN_DENIAL
};
