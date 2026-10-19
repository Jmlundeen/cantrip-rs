use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Duelist of the Mind's power is equal to the number of cards you've drawn this turn" (CR 604.3, layer 7a): a
/// characteristic-defining ability, counting every card its controller has drawn since the turn began, in any step and on
/// either player's turn (the history of the turn, CR 121).
fn power_is_cards_drawn_this_turn(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let controller = q.state().obj(source).controller;
    chars.power = Some(q.history().drawn(controller) as i16);
}

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(Layer::PtDefine, power_is_cards_drawn_this_turn, "Duelist of the Mind's power is equal to the number of cards you've drawn this turn.").lost_pt(0, 3)],
    abilities: &[triggered(triggers::YOU_COMMIT_A_CRIME, effects::may_draw_then_discard, "Whenever you commit a crime, you may draw a card. If you do, discard a card. This ability triggers only once each turn. (Targeting opponents, anything they control, and/or cards in their graveyards is a crime.)")
        .triggers_each_turn(1)],
    ..oracle::DUELIST_OF_THE_MIND
};
