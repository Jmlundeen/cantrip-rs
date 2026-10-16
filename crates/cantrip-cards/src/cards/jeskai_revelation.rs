use cantrip_api::*;
use cantrip_mechanics::tokens::{self, MONK};
use cantrip_mechanics::{damage_target, effects, targets};
use cantrip_oracle as oracle;

/// "Return target spell or permanent to its owner's hand. Jeskai Revelation deals 4 damage to any target. Create two 1/1
/// white Monk creature tokens with prowess. Draw two cards. You gain 4 life." A spell leaves the stack for its owner's
/// hand without being countered; a permanent is returned (a token ceases to exist). Each target is checked on its own
/// (CR 608.2b): one gone does not stop the other or the rest of the spell.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(object) = targets::object(g, item, 0) {
        effects::return_spell_or_permanent_to_hand(g, object);
    }
    damage_target(g, item, 1, 4);
    tokens::create(g, item.controller, &MONK, 2)?;
    g.draw(item.controller, 2);
    g.gain_life(item.controller, 4);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[ targets::SPELL_OR_PERMANENT.labelled("Return target spell or permanent to its owner's hand"), targets::ANY.labelled("Deal 4 damage to any target"), ], effect),
    ..oracle::JESKAI_REVELATION
};

pub static EXTRA: &[&CardDef] = &[&MONK];
