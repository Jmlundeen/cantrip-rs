use cantrip_api::*;
use cantrip_mechanics::{empower, triggers};
use cantrip_oracle as oracle;

/// "When Way of the Mind Sculptor enters, empower Jace 5."
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 5)
}

/// "Whenever you activate a loyalty ability, if you removed two or more loyalty counters to activate it" (CR 606.4, 603.4):
/// the loyalty cost was a -N of two or more, or a -X announced as two or more. A fixed cost is read from the ability; an X
/// from the activation just put on the stack, where triggers are collected (CR 602.2b).
fn you_removed_two_or_more_loyalty_counters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    triggers::loyalty_ability_you_activated(q, me, event)?;
    let Event::AbilityActivated { source, ability, .. } = *event else { return None };
    let cost = q.activated_ability(ability)?.cost.loyalty?;
    let removed = if cost == LOYALTY_MINUS_X {
        q.state().stack.iter().rev().find(|i| i.kind == StackKind::Activated && i.object == source && i.ability == ability)?.x
    } else {
        (-i16::from(cost)).max(0) as u16
    };
    if removed >= 2 { Fired::PLAIN } else { None }
}

/// "draw a card."
fn draw_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, empower_jace, "When Way of the Mind Sculptor enters, empower Jace 5."),
        triggered(
            when_asked(EventMask::ABILITY_ACTIVATED, you_removed_two_or_more_loyalty_counters),
            draw_a_card,
            "Whenever you activate a loyalty ability, if you removed two or more loyalty counters to activate it, draw a card.",
        ),
    ],
    ..oracle::WAY_OF_THE_MIND_SCULPTOR
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
