use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Draw three cards, then put two cards from your hand on top of your library in any order."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::draw_then_put_two_back(g, item, 3)
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::BRAINSTORM };
