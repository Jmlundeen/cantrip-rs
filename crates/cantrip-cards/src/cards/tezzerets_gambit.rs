use cantrip_api::*;
use cantrip_oracle as oracle;

/// Draw two cards, then proliferate (CR 701.34a).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    g.proliferate(item.controller, item.object)
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::TEZZERETS_GAMBIT };
