use cantrip_api::*;
use cantrip_mechanics::statics::more_than_one_card_drawn_this_turn;
use cantrip_oracle as oracle;

/// "Each player can't draw more than one card each turn" (CR 121.2b): every player, its controller included. A draw past
/// the first is not made, and no replacement effect replaces it (CR 614.17c, its ruling); a choice to draw two or more
/// can't be taken (CR 121.2b).
fn each_player(q: &dyn Query, _: &Asker, player: PlayerId, n: u32) -> bool {
    more_than_one_card_drawn_this_turn(q, player, n)
}

pub static CARD: CardDef = CardDef { cant_draw: &[each_player], ..oracle::SPIRIT_OF_THE_LABYRINTH };
