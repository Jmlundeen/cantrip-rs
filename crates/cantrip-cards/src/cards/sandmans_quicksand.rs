use cantrip_api::*;
use cantrip_mechanics::{effects, mayhem, pump};
use cantrip_oracle as oracle;

/// "All creatures get -2/-2 until end of turn. If this spell's mayhem cost was paid, creatures your opponents control get
/// -2/-2 until end of turn instead." The creatures on the battlefield as the spell resolves (CR 611.2c), those an opponent
/// controls when a mayhem cost was paid, the printed one or the one Green Goblin gives (CR 702.187a-b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !mayhem::paid(g, item.object) {
        effects::pump_all_creatures(g, item.object, -2, -2);
        return Ok(());
    }
    let creatures: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.controller(id) != item.controller)
        .collect();
    instruction(g, |g| {
        for id in creatures {
            pump(g, item.object, id, -2, -2);
        }
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[mayhem::way(mana("{3}{B}"), "Mayhem {3}{B}")],
    ..oracle::SANDMANS_QUICKSAND
};
