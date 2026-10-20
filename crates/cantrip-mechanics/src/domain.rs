//! Domain (an ability word, CR 207.2c): count the basic land types among lands a player controls (CR 305.6).
//! Fblthp, Knows the Way, Territorial Kavu and Leyline Binding use this same count.

use cantrip_api::*;

/// The number of distinct basic land types among lands `player` controls, using current characteristics.
pub fn count(q: &dyn Query, player: PlayerId) -> u8 {
    count_types(q.state(), player, |id| q.chars(id))
}

/// The same count during a continuous effect, using the characteristics from the preceding layers.
pub fn count_in_layers(q: &dyn LayerQuery, player: PlayerId) -> u8 {
    count_types(q.state(), player, |id| q.chars(id))
}

fn count_types(state: &GameState, player: PlayerId, chars: impl Fn(ObjectId) -> Chars) -> u8 {
    let mut types = SubtypeSet::NONE;
    for &id in state.battlefield() {
        if state.obj(id).controller != player {
            continue;
        }
        let land = chars(id);
        if land.is(CardType::Land) {
            for kind in land.subtypes.iter().filter(|kind| kind.basic_mana().is_some()) {
                types = types.with(kind);
            }
        }
    }
    types.iter().count() as u8
}
