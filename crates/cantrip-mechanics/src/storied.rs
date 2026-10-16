//! Storied (CR 702.195): "Any time you control three or more permanents that are artifacts, Sagas, and/or legendary and you
//! don't have an enduring story, you have an enduring story for the rest of the game." The keyword is a card's
//! `storied: true`, which the rules read to give the designation (`Player::enduring_story`, CR 702.195b); what is here is
//! "as long as you have an enduring story", which Fíli the Pathfinder, Óin the Brave, Ori, Keeper of Songs, Bombur,
//! Gentle Dreamer, Thorin Oakenshield, Kíli the Resourceful and Bifur, Melodic Rider print.

use cantrip_api::*;

/// "As long as you have an enduring story" (CR 702.195b): the controller of `source` has the designation.
pub fn you_have_an_enduring_story(s: &GameState, source: ObjectId) -> bool {
    s.player(s.obj(source).controller).enduring_story
}

/// "As long as you have an enduring story, [this creature] gets ..." as a static ability's `StaticDef::affects`: the
/// permanent itself, while its controller has an enduring story. Óin the Brave and Ori, Keeper of Songs print it.
pub fn itself_with_an_enduring_story(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && you_have_an_enduring_story(q.state(), source)
}
