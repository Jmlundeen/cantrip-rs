//! Max speed (CR 702.178): "Max speed — [Ability]" means "As long as your speed is 4, this object has '[Ability].'" A
//! static ability of the permanent that grants it the ability its card lists first in `delayed` (CR 613.1f, layer 6),
//! so the permanent does not have that ability at all below max speed - it can't be activated and it doesn't trigger -
//! and an effect that removes the permanent's abilities removes the grant with them. "Your" is its controller's
//! (CR 109.5), so a permanent whose control changes has the ability by its new controller's speed. Perilous Snare prints
//! it. A granted ability that functions from another zone (CR 702.178b) waits for a card that prints one.

use cantrip_api::*;

use crate::start_your_engines::has_max_speed;

/// "As long as your speed is 4" (CR 702.178a): the permanent itself, while its controller has max speed. Perilous Snare's granted ability and Burnout Bashtronaut's double strike use it.
pub fn itself_at_max_speed(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, _: &Chars) -> bool {
    source == affected && has_max_speed(view.state(), view.state().obj(source).controller)
}

/// "Max speed — [Ability]" (CR 702.178a), for the card's `statics`: the card's delayed ability 0 is the granted one, and
/// `text` is the whole line as printed.
pub const fn max_speed(text: &'static str) -> StaticDef {
    static_ability(Layer::Ability, itself_at_max_speed, crate::statics::grant_delayed_ability, text)
}
