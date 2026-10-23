use cantrip_api::*;
use cantrip_mechanics::tokens::{self, investigate};
use cantrip_oracle as oracle;

/// "Draw a card. Investigate.": draw, then investigate (CR 701.16a); the sentence ends after the draw (CR 608.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    g.end_sentence();
    investigate(g, item.controller)
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::DEDUCE };

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
