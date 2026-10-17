use cantrip_api::*;
use cantrip_mechanics::statics::opponents_cant_draw_more_than_one_card_each_turn;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

static ASK: Question = Question::new("Draw", "draw a card?", "", "Draw", "Don't draw");

/// "Whenever you or a permanent you control becomes the target of a spell or ability an opponent controls": a
/// permanent is one on the battlefield. Each thing named is its own event, so each triggers it (CR 603.2).
fn targeted(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    let you = q.controller(source);
    match event {
        Event::BecameTarget { target, controller, .. } if *controller != you => {
            let yours = match target {
                Target::Player(p) => *p == you,
                Target::Object(id) => targets::permanent(q, *id) && q.controller(*id) == you,
                _ => false,
            };
            if yours { Fired::PLAIN } else { None }
        }
        _ => None,
    }
}

/// "You may draw a card." Not offered when the player can't draw a card (CR 121.3): an opponent's Leovold once this turn's
/// draw is used up.
fn may_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::may_draw(g, item.controller, item.object, 1, &ASK)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::BECAME_TARGET, targeted), may_draw, "Whenever you or a permanent you control becomes the target of a spell or ability an opponent controls, you may draw a card.")],
    // "Each opponent can't draw more than one card each turn" (CR 121.2b): each draw past the first is not made, and no
    // replacement effect replaces it (CR 614.17c).
    cant_draw: &[opponents_cant_draw_more_than_one_card_each_turn],
    ..oracle::LEOVOLD_EMISSARY_OF_TREST
};
