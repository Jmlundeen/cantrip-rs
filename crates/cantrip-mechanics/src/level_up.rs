//! Level up [cost] (CR 702.87a): "[Cost]: Put a level counter on this permanent. Activate only as a sorcery." Hexdrinker
//! prints it. The level symbols' abilities (CR 711) are the card's own statics, read from the level counters.

use cantrip_api::*;

fn add_level(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.is_on_battlefield(item.object) && g.obj(item.object).became.is_none() {
        g.add_counters(item.object, CounterKind::Level, 1);
    }
    Ok(())
}

/// The number of level counters on `source`, as the layers see it.
pub fn levels(view: &dyn LayerQuery, source: ObjectId) -> i16 {
    view.state().obj(source).counters(CounterKind::Level)
}

pub const fn ability(cost: ManaCost, text: &'static str) -> AbilityDef {
    cantrip_api::ability(Cost::mana(cost), add_level, text)
        .sorcery_speed()
}
