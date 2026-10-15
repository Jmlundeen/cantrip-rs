//! Foretell (CR 702.143a): "During your turn, you may pay {2} and exile this card from your hand face down. Cast it on a
//! later turn for its foretell cost." A special action (CR 116.2): no stack, nothing to respond to. Delayed Blast
//! Fireball and Saw It Coming print it.
//!
//! A card with foretell sets `foretell: Some(cost)` and lists `ABILITY`. The card is exiled face down (CR 702.143a; only its owner may look at it) and its owner may cast it from
//! there for the foretell cost on any later turn, with the card's ordinary timing.

use cantrip_api::*;

fn foretell(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    let cost = g.card(g.obj(card).card).foretell;
    let owner = g.obj(card).owner;
    let exiled = g.exile_face_down(&[card], None)[0];
    if g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.allow_play(exiled, owner, PlayGrant { instead: cost, later_turns: true, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static ABILITY: AbilityDef = AbilityDef::Activated(ActivatedDef {
    text: "Foretell {2} (During your turn, you may pay {2} and exile this card from your hand face down. Cast it on a later turn for its foretell cost.)",
    cost: Cost { mana: mana(2, 0, 0, 0, 0, 0), ..Cost::NONE },
    timing: Timing::OwnTurn,
    from: ZoneKind::Hand,
    effect: foretell,
    special_action: true,
    ..ActivatedDef::BLANK
});
