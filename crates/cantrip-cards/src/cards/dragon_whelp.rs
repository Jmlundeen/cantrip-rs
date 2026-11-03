use cantrip_api::*;
use cantrip_mechanics::{delayed, effects};
use cantrip_oracle as oracle;

/// Each resolution counts this object's activations of this ability this turn (CR 608.2c, 603.7).
fn pump_and_schedule(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::this_gets::<1, 0>(g, item)?;
    g.end_sentence();
    let activations = g.history().this_turn().iter().filter(|r| {
        matches!(r.event, Event::AbilityActivated { source, ability, .. }
            if source == item.object && ability == item.ability)
    }).count();
    if activations >= 4 {
        delayed::at_next_end_step(g, item, 0, Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{R}"), pump_and_schedule, "{R}: This creature gets +1/+0 until end of turn. If this ability has been activated four or more times this turn, sacrifice this creature at the beginning of the next end step.")],
    delayed: &[delayed_trigger(effects::sacrifice_self, "If this ability has been activated four or more times this turn, sacrifice this creature at the beginning of the next end step.")],
    ..oracle::DRAGON_WHELP
};
