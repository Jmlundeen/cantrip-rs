use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::YOUR_END_STEP};
use cantrip_oracle as oracle;

/// "At the beginning of your end step, put a time counter on Kes, then draw X cards, where X is the number of time
/// counters on her. Then if she has nine or more time counters on her, sacrifice her." (CR 603.2, 608.2c, 121.1,
/// 122.1, 701.21): the counter goes on first, so X counts it; the draw is its own sentence; and the sacrifice is asked
/// of the counters Kes has once the cards are drawn - the last of three instructions in the order written (CR 608.2c).
fn end_step(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let kes = item.object;
    g.add_counters(kes, CounterKind::Time, 1);
    let drawn = g.obj(kes).counters(CounterKind::Time).max(0) as u32;
    g.end_sentence();
    effects::draw(g, item, drawn)?;
    g.end_sentence();
    if g.obj(kes).counters(CounterKind::Time) >= 9 {
        g.sacrifice(item.controller, &[kes]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        end_step,
        "At the beginning of your end step, put a time counter on Kes, then draw X cards, where X is the number of time counters on her. Then if she has nine or more time counters on her, sacrifice her.",
    )],
    ..oracle::KES_EVOLVING_TELEPATH
};
