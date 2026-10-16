use cantrip_api::*;
use cantrip_mechanics::web_slinging;
use cantrip_oracle as oracle;

/// "Target instant spell, sorcery spell, or triggered ability" (CR 115.1): an instant or sorcery spell on the stack, or a
/// triggered ability there. An activated ability is not one of them.
fn instant_sorcery_or_trigger(q: &dyn Query, _: &Asker, t: Target) -> bool {
    match t {
        Target::Ability(serial) => q.state().stack.iter().any(|i| i.serial == serial && i.kind == StackKind::Triggered),
        Target::Object(id) => {
            q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id) && {
                let c = q.chars(id);
                c.is(CardType::Instant) || c.is(CardType::Sorcery)
            }
        }
        _ => false,
    }
}

/// "Counter target instant spell, sorcery spell, or triggered ability." (CR 701.6a, 701.6b) Its one target is still legal
/// as it resolves, or it would not resolve at all (CR 608.2b): still on the stack.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    match item.targets[0] {
        Target::Ability(serial) => {
            let at = g.state().stack.iter().position(|i| i.serial == serial).expect("a legal target is on the stack");
            let countered = g.state().stack[at];
            g.counter_item(countered.object, at);
        }
        spell => g.counter(spell.object().expect("an instant or sorcery spell")),
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(instant_sorcery_or_trigger)], effect),
    ways: &[web_slinging::way(mana("{U}"), "Web-slinging {U}")],
    ..oracle::SPIDER_SENSE
};
