//! The Lhurgoyf size: "power is equal to the number of card types among cards in all graveyards and its toughness is
//! equal to that number plus 1" (CR 604.3, 205.2a). Tarmogoyf and Barrowgoyf print it word for word.

use cantrip_api::*;

fn apply(q: &dyn LayerQuery, _: ObjectId, chars: &mut Chars) {
    let state = q.state();
    let mut seen = TypeSet::NONE;
    for player in state.seats() {
        for &card in state.zone(Zone::graveyard(player)) {
            for t in q.chars(card).types.iter() {
                seen = seen.with(t);
            }
        }
    }
    let types = seen.iter().count() as i16;
    chars.power = Some(types);
    chars.toughness = Some(types + 1);
}

/// A characteristic-defining ability (`own`), so it applies in every zone the card is in.
pub const SIZE: StaticDef = characteristic_defining(Layer::PtDefine, apply, "This creature's power is equal to the number of card types among cards in all graveyards and its toughness is equal to that number plus 1.");
