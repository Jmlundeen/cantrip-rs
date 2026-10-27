use cantrip_api::*;
use cantrip_mechanics::{empower, library, targets, triggers};
use cantrip_oracle as oracle;

/// "The owner of up to one other target nonland permanent puts it on their choice of the top or bottom of their
/// library" The owner chooses; yes means top.
fn put_in_library(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    library::owner_puts_on_top_or_bottom(g, item.object, permanent)
}

/// "Empower Jace 1" (CR 701.71a).
fn empower_jace(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    empower::jace(g, item.controller, item.object, 1)
}

/// The first noncreature spell is a question of the entire turn's history, not the first trigger
/// after this enchantment arrived (CR 603.2).
fn first_noncreature(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    triggers::noncreature_spell_you_cast(q, me, event)?;
    let Event::SpellCast { spell, player, .. } = event else { return None };
    if cantrip_mechanics::history::first_noncreature_spell_this_turn(q, *spell, *player) {
        Fired::PLAIN
    } else {
        None
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            put_in_library,
            "When this enchantment enters, the owner of up to one other target nonland permanent puts it on their choice of the top or bottom of their library.",
        )
        .targets(&[up_to_one(targets::other_nonland_permanent_filter)]),
        triggered(
            when_asked(EventMask::SPELL_CAST, first_noncreature),
            empower_jace,
            "Whenever you cast your first noncreature spell each turn, empower Jace 1. (Put a loyalty counter on a Jace token you control. If you don't control one, first create a blue Jace planeswalker token with \"[-1]: Surveil 1\" and \"[-3]: Draw a card.\")",
        ),
    ],
    ..oracle::PLAN_FOR_ALL_OUTCOMES
};

pub static EXTRA: &[&CardDef] = &[&empower::JACE];
