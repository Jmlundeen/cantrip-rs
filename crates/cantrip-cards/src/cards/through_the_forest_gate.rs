use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Look at the top twenty cards of your library, put any number of land cards from among them onto the battlefield tapped, then
/// shuffle. You gain 8 life." (CR 401.2, 608.2c, 701.24, 119.3): a library of fewer than twenty shows what it has (CR 609.3); the
/// player sees every card and may take any number of the lands (none included), which enter together, tapped (CR 110.5b,
/// 603.6a); the rest stay in the library, which is then shuffled whether or not a land was taken. The life follows the
/// shuffle, a sentence of its own.
fn gate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    effects::look_at_top_put_any_lands_onto_the_battlefield_tapped_then_shuffle(g, you, item.object, 20)?;
    g.end_sentence();
    g.gain_life(you, 8);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], gate), ..oracle::THROUGH_THE_FOREST_GATE };
