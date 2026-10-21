use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever Ruric Thar becomes the target of a spell or ability an opponent controls, draw a card" (CR 603.2): one
/// trigger each time this creature becomes a target, so two spells or abilities that target it draw two cards. A single
/// spell naming it in two target slots makes it the target once (CR 115.3), so that draws one, as `ops::became_targets`
/// records it. Only an opponent's spell or ability counts; its own controller's do not (CR 109.5).
fn targeted_by_an_opponent(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::BecameTarget { target: Target::Object(t), controller, .. }
            if *t == source && *controller != q.controller(source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    // Flying is a core keyword in the content JSON (CR 702.9). "Prowess, prowess" is two instances of the keyword
    // ability (CR 702.108a), so one noncreature spell triggers it twice and it gets +1/+1 twice (the reminder text).
    abilities: &[
        triggers::PROWESS,
        triggers::PROWESS,
        triggered(
            when(EventMask::BECAME_TARGET, targeted_by_an_opponent),
            |g, item| effects::draw(g, item, 1),
            "Whenever Ruric Thar becomes the target of a spell or ability an opponent controls, draw a card.",
        ),
    ],
    ..oracle::RURIC_THAR_BIOMAGUS
};
