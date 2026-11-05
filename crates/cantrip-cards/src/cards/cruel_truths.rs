use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Surveil 2, then draw two cards. You lose 2 life." (CR 701.25, 121.1, 119.3): the first sentence's surveil and draw are done in
/// order; the life is lost as the second sentence.
fn surveil_draw_lose(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::surveil(g, item, 2)?;
    effects::draw(g, item, 2)?;
    g.end_sentence();
    g.lose_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], surveil_draw_lose), ..oracle::CRUEL_TRUTHS };
