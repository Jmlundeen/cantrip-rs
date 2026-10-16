use cantrip_api::*;
use cantrip_mechanics::{damage_divided, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast a noncreature spell, create a tapped Treasure token and put a plan counter on this enchantment."
/// (CR 111.10a, 122.6) The counter goes only on the enchantment that triggered, if it is still there (CR 400.7).
fn treasure_and_plan(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_tapped_treasure(g, item.controller)?;
    g.add_counters(item.object, CounterKind::Plan, 1);
    Ok(())
}

/// "When the fourth plan counter is put on this enchantment, sacrifice it. When you do, ..." (CR 701.21a, 603.12): the
/// reflexive ability triggers only if the enchantment was sacrificed, and its targets and the division of the damage are
/// chosen as it is put on the stack (CR 601.2d).
fn sacrifice_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.sacrifice(&[item.object]).is_empty() {
        return Ok(());
    }
    let this = g.card_id(&CARD);
    g.reflexive_trigger(item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

/// "It deals 7 damage divided as you choose among one or two targets": the sacrificed enchantment, as it last existed,
/// is the source (CR 608.2h); a target that has become illegal is left out with its share (CR 608.2b).
fn seven_damage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_divided(g, item);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::NONCREATURE_SPELL_YOU_CAST, treasure_and_plan, "Whenever you cast a noncreature spell, create a tapped Treasure token and put a plan counter on this enchantment."),
        triggered(
            when(EventMask::COUNTERS, triggers::nth_counter_put_on_this::<{ CounterKind::Plan as u8 }, 4>),
            sacrifice_it,
            "When the fourth plan counter is put on this enchantment, sacrifice it. When you do, it deals 7 damage divided as you choose among one or two targets.",
        ),
    ],
    delayed: &[delayed_trigger(seven_damage, "When you do, it deals 7 damage divided as you choose among one or two targets.")
        .targets(&[targets::ANY.between(1, 2)])
        .divide(Divide { total: 7, of_x: None })],
    ..oracle::DEATH_TO_OUR_ENEMIES
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
