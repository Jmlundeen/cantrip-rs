use cantrip_api::*;
use cantrip_mechanics::{spent, triggers};
use cantrip_oracle as oracle;

/// Opus (an ability word, CR 207.2c): "Whenever you cast an instant or sorcery spell, put a +1/+1 counter on this creature. If five
/// or more mana was spent to cast that spell, add an amount of {R} equal to this creature's power." The counter goes on first, so
/// it counts; the spell's mana spent (CR 601.2h) is read from it even once it has been countered or has left the stack (CR 608.2h);
/// a spell cast without paying its mana cost has had none spent. If this creature has left the battlefield by the time the
/// trigger resolves, it gets no counter (CR 400.7) and its power is the last it had there (CR 608.2h), which may be nothing or less
/// (the amount of mana is never below zero).
fn opus(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    if spent::five_or_more_mana_spent_on_the_spell(g, item) {
        let power = g.chars(item.object).power.unwrap_or(0).max(0) as u32;
        g.add_mana(item.controller, Mana::R, power);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::INSTANT_OR_SORCERY_YOU_CAST,
        opus,
        "Opus \u{2014} Whenever you cast an instant or sorcery spell, put a +1/+1 counter on this creature. If five or more mana was spent to cast that spell, add an amount of {R} equal to this creature's power.",
    )],
    ..oracle::MOLTEN_CORE_MAESTRO
};
