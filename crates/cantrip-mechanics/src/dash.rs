//! Dash [cost] (CR 702.109a): "You may cast this card from your hand for its dash cost. If you do, it gains haste, and
//! it's returned from the battlefield to its owner's hand at the beginning of the next end step." Death Greeter's
//! Champion and Ragavan, Nimble Pilferer print it.
//!
//! A card with dash lists `way(cost, text)` as its only alternative cost, `RIDERS` as its `end_step_riders`, `RETURN`
//! among its `delayed` abilities as the first, and `HASTE` among its `statics`: the way casts it, and the core creates
//! the delayed ability without the stack as the spell is cast for it (CR 608.3g, 603.7a), about the permanent the
//! spell becomes.

use cantrip_api::*;

/// The alternative cost of dash: cast from the hand for `cost` (CR 118.9). It is the card's first way.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    WayDef { text, cost: Cost::mana(cost), from: ZoneKind::Hand, exile_after: false, condition: None, instant_speed: false }
}

/// Dash's delayed ability is the card's first and its way the first: the end-step rider pairs them.
pub const RIDERS: &[(u8, u8)] = &[(1, 0)];

fn dashed(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    affected == source && view.state().obj(source).cast_way == 1
}

/// "If this permanent's dash cost was paid, it has haste", read on every pass.
pub const HASTE: StaticDef = StaticDef {
    text: "As long as this permanent's dash cost was paid, it has haste.",
    layer: Layer::Ability,
    affects: dashed,
    apply: crate::statics::have_haste,
    ..StaticDef::BLANK
};

/// Return it to its owner's hand: the permanent the dashed spell became (CR 702.109a, 608.3g), if it is still there.
fn return_to_hand(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(spell) = item.subject else { return Ok(()) };
    let mut now = spell;
    while let Some(next) = g.obj(now).became {
        now = next;
    }
    if g.obj(now).zone.is(ZoneKind::Battlefield) {
        g.return_to_hand(&[now]);
    }
    Ok(())
}

pub const RETURN: AbilityDef = delayed_trigger(return_to_hand, "Return it to its owner's hand.");
