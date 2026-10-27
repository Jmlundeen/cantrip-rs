use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, triggers};
use cantrip_oracle as oracle;

/// "put a quest counter on this enchantment. Draw a card.
/// Then discard a card unless this enchantment has three or more quest counters on it."
fn quest(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Quest, 1);
    g.draw(item.controller, 1);
    if g.obj(item.object).counters(CounterKind::Quest) < 3 {
        discard_chosen(g, item.controller, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::NONCREATURE_SPELL_YOU_CAST, quest,
                           "Whenever you cast a noncreature spell, put a quest counter on this enchantment. Draw a card. Then discard a card unless this enchantment has three or more quest counters on it.")
    ],
    ..oracle::ANTHROPOLOGICAL_MISSION
};
