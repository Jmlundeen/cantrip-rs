use cantrip_api::*;
use cantrip_mechanics::{creature_types_among_non_borg, triggers};
use cantrip_oracle as oracle;

/// "Federation — At the beginning of combat on your turn, draw a card for every five creature types among non-Borg
/// creatures you control. If you draw three or more cards this way, put two +1/+1 counters on each creature you control."
/// (CR 205.3m, 121.1, 122.1) The types are counted once as the ability resolves and give one card for each complete five
/// (CR 107.1a); the cards counted after it are the ones actually drawn, so a replacement effect that stops a draw counts
/// for less (CR 614.1). The two counters go on every creature at once (CR 101.4).
fn draw_for_creature_types(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let types = creature_types_among_non_borg(g, item.object, item.controller);
    let before = g.history().drawn(item.controller);
    g.draw(item.controller, (types / 5) as u32);
    let drawn = g.history().drawn(item.controller) - before;
    if drawn >= 3 {
        let counters: Vec<(ObjectId, CounterKind, i32)> = g
            .permanents_of(item.controller)
            .into_iter()
            .filter(|&id| g.chars(id).is(CardType::Creature))
            .map(|id| (id, CounterKind::PlusOne, 2))
            .collect();
        g.add_counters_together(&counters);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::BEGINNING_OF_YOUR_COMBAT,
        draw_for_creature_types,
        "Federation — At the beginning of combat on your turn, draw a card for every five creature types among non-Borg creatures you control. If you draw three or more cards this way, put two +1/+1 counters on each creature you control.",
    )],
    ..oracle::SAREK_FEDERATION_AMBASSADOR
};
