use cantrip_api::*;
use cantrip_mechanics::{statics::your_creatures, top_of_library};
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
        g.return_to_hand_revealed(&[top]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    delayed: &[triggered(when(EventMask::BECAME_TARGET, targeted), reveal_top, "Whenever this creature becomes the target of a spell or ability, reveal the top card of your library. If it's a land card, put it onto the battlefield. Otherwise, put it into your hand. This ability triggers only twice each turn.")
        .triggers_each_turn(2)],
    statics: &[static_ability(Layer::Ability, your_creatures, cantrip_mechanics::statics::grant_delayed_ability, "Creatures you control have \"Whenever this creature becomes the target of a spell or ability, reveal the top card of your library. If it's a land card, put it onto the battlefield. Otherwise, put it into your hand. This ability triggers only twice each turn.\"")],
    ..oracle::NADU_WINGED_WISDOM
};
