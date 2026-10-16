use cantrip_api::*;
use cantrip_mechanics::{shuffle_hands_and_graveyards_in_and_draw_seven, untap};
use cantrip_oracle as oracle;

/// Exile Time Spiral. Each player shuffles their hand and graveyard into their library, then draws seven cards. You
/// untap up to six lands. The spell exiles itself as its first instruction (CR 608.2m), so it is not in a graveyard to
/// be shuffled in, and leaves the stack for nowhere more when it finishes.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.exile(item.object);
    shuffle_hands_and_graveyards_in_and_draw_seven(g);
    untap::up_to_lands(g, item.object, item.controller, 6)
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::TIME_SPIRAL };
