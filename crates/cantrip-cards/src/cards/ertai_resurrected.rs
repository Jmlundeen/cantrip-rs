use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// "Target spell, activated ability, or triggered ability" (CR 115.1): a spell or an ability on the stack.
fn spell_or_ability(q: &dyn Query, _: &Asker, t: Target) -> bool {
    match t {
        Target::Ability(_) => true,
        Target::Object(id) => q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id),
        _ => false,
    }
}

/// "Another target creature or planeswalker."
fn another_creature_or_planeswalker(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source && targets::permanent(q, id) && {
        let c = q.chars(id);
        c.is(CardType::Creature) || c.is(CardType::Planeswalker)
    })
}

/// Mode 0: counter the target, then its controller draws a card. The draw is a separate instruction (CR 701.5) and
/// still happens when the spell can't be countered; the controller is read before the counter.
fn counter_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let controller = match item.targets[0] {
        Target::Object(spell) => {
            let controller = g.controller(spell);
            g.counter(spell);
            controller
        }
        Target::Ability(serial) => {
            let Some(at) = g.state().stack.iter().position(|i| i.serial == serial) else { return Ok(()) };
            let countered = g.state().stack[at];
            g.counter_item(countered.object, at);
            countered.controller
        }
        _ => return Ok(()),
    };
    g.draw(controller, 1);
    Ok(())
}

/// Mode 1: destroy the target, then its controller draws a card (still drawn when it is indestructible, CR 701.7b).
fn destroy_and_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = cantrip_mechanics::targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(permanent);
    g.destroy(permanent);
    g.draw(controller, 1);
    Ok(())
}

static MODAL: ModalDef = ModalDef {
    modes: &[
        ModeDef {
            text: "Counter target spell, activated ability, or triggered ability. Its controller draws a card.",
            targets: &[TargetSpec { min: 1, max: 1, filter: spell_or_ability }],
            effect: counter_and_draw,
            cost: ManaCost::ZERO,
        },
        ModeDef {
            text: "Destroy another target creature or planeswalker. Its controller draws a card.",
            targets: &[TargetSpec { min: 1, max: 1, filter: another_creature_or_planeswalker }],
            effect: destroy_and_draw,
            cost: ManaCost::ZERO,
        },
    ],
    min: 0,
    max: 1,
    escalate: None,
    repeat: false, allowed: None,
};

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When Ertai Resurrected enters, choose up to one \u{2014} Counter target spell, activated ability, or triggered ability. Its controller draws a card; or destroy another target creature or planeswalker. Its controller draws a card.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        modal: Some(&MODAL),
        ..TriggeredDef::BLANK
    })],
    ..oracle::ERTAI_RESURRECTED
};
