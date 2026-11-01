use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// "Counter target spell. / Draw a card, then mill three cards. / Untap target land." (CR 608.2c): three sentences, each its
/// own instruction, done in order; a target that became illegal is skipped (CR 608.2b) and the rest still happens.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    g.end_sentence();
    g.draw(item.controller, 1);
    g.end_sentence();
    g.mill(item.controller, 3);
    g.end_sentence();
    if let Some(land) = targets::object(g, item, 1) {
        g.untap(land);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::SPELL, targets::LAND], effect), ..oracle::SOKKAS_HAIKU };
