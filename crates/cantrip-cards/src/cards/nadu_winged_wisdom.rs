use cantrip_api::*;
use cantrip_mechanics::top_of_library;
use cantrip_oracle as oracle;

/// "Whenever this creature becomes the target of a spell or ability" (CR 115.1): anyone's, its controller's own included.
fn targeted(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::BecameTarget { target: Target::Object(id), .. } if *id == source => Fired::PLAIN,
        _ => None,
    }
}

/// Reveal the top card of your library. If it's a land card, put it onto the battlefield. Otherwise, put it into your hand.
/// An empty library does nothing.
fn reveal_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let Some(&top) = top_of_library(g, you, 1).first() else { return Ok(()) };
    g.reveal(you, &[top]);
    if g.chars(top).is(CardType::Land) {
        g.put_onto_battlefield(top, you, false)?;
    } else {
        g.return_to_hand(&[top]);
    }
    Ok(())
}

/// Creatures you control, Nadu itself among them: "creatures you control have ..." is not "other".
fn your_creatures(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Creature) && view.state().obj(affected).controller == view.state().obj(source).controller
}

/// Gives the creature Nadu's own delayed ability 0 (CR 613.1f). The creature is the ability's source, so the limit of
/// two triggerings each turn (CR 603.2h) is counted for each creature on its own.
fn grant(view: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let o = view.state().obj(source);
    let card = o.copy.map(|c| c.card).unwrap_or(o.card);
    if let Some(slot) = chars.abilities.iter().position(|a| a.is_none()) {
        chars.abilities[slot] = Some(AbilityRef::delayed(card, 0));
    }
}

pub static CARD: CardDef = CardDef {
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever this creature becomes the target of a spell or ability, reveal the top card of your library. If it's a land card, put it onto the battlefield. Otherwise, put it into your hand. This ability triggers only twice each turn.",
        listens: EventMask::BECAME_TARGET,
        matches: targeted,
        effect: reveal_top,
        triggers_each_turn: 2,
        ..TriggeredDef::BLANK
    })],
    statics: &[StaticDef {
        text: "Creatures you control have \"Whenever this creature becomes the target of a spell or ability, reveal the top card of your library. If it's a land card, put it onto the battlefield. Otherwise, put it into your hand. This ability triggers only twice each turn.\"",
        layer: Layer::Ability,
        affects: your_creatures,
        apply: grant,
        ..StaticDef::BLANK
    }],
    ..oracle::NADU_WINGED_WISDOM
};
