use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever a player casts a noncreature spell, put a +1/+1 counter on Lieutenant Commander Data" (CR 603.2): the player
/// who cast it does not matter, and Data's controller puts the counter as the trigger resolves, on Data if it is still
/// the permanent that triggered (CR 113.7a, 400.7).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::A_PLAYER_CASTS_A_NONCREATURE_SPELL,
            grow,
            "Whenever a player casts a noncreature spell, put a +1/+1 counter on Lieutenant Commander Data.",
        ),
        ability(
            cost("{1}").and(remove_counters_among(3, targets::filters::creature)),
            |g, item| effects::draw(g, item, 1),
            "{1}, Remove three counters from among creatures you control: Draw a card.",
        ),
    ],
    ..oracle::LIEUTENANT_COMMANDER_DATA
};
