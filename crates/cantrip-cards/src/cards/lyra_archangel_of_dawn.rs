use cantrip_api::*;
use cantrip_mechanics::triggers::YOU_GAIN_LIFE;
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on each Angel you control": the Angels you control as the ability resolves (CR 608.2c), Lyra herself
/// among them if she is still there. "Angel" is a word of her text (CR 612).
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let angel = g.words(item.object).subtype(Subtype::Angel);
    let angels: Vec<ObjectId> = g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).has_subtype(angel)).collect();
    for id in angels {
        g.add_counters(id, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(YOU_GAIN_LIFE, counters, "Whenever you gain life, put a +1/+1 counter on each Angel you control.")],
    ..oracle::LYRA_ARCHANGEL_OF_DAWN
};
