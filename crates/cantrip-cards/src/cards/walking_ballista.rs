use cantrip_api::*;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "This creature enters with X +1/+1 counters on it", for the X it was cast with ({X}{X} is paid twice for it);
/// put onto the battlefield without being cast, X is 0.
fn enters_with_x_counters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::PlusOne as usize] += entering.x as i16;
    Ok(())
}

/// {4}: Put a +1/+1 counter on this creature.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if is_live(g, item.object) && g.is_on_battlefield(item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

/// It deals 1 damage to any target (the counter is already gone: the cost).
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_x_counters),
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{4}: Put a +1/+1 counter on this creature.",
            cost: Cost::mana(mana(4, 0, 0, 0, 0, 0)),
            effect: grow,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "Remove a +1/+1 counter from this creature: It deals 1 damage to any target.",
            cost: Cost { remove_counters: Some((CounterKind::PlusOne, 1)), ..Cost::NONE },
            targets: &[targets::ANY],
            effect: ping,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::WALKING_BALLISTA
};
