use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Put X +1/+1 counters on each creature you control, where X is this creature's power": X is worked out once as the
/// ability resolves, from last known information if the Ouroboroid has gone (CR 608.2h); a power of zero or less puts
/// nothing anywhere.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let source = item.object;
    let power = if targets::permanent(g, source) {
        g.chars(source).power.unwrap_or(0)
    } else {
        g.state().last_known(source).and_then(|k| k.power).unwrap_or(0)
    };
    if power <= 0 {
        return Ok(());
    }
    let creatures: Vec<ObjectId> =
        g.permanents_of(item.controller).into_iter().filter(|&id| g.chars(id).is(CardType::Creature)).collect();
    for id in creatures {
        g.add_counters(id, CounterKind::PlusOne, power);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "At the beginning of combat on your turn, put X +1/+1 counters on each creature you control, where X is Ouroboroid's power.",
        listens: EventMask::STEP_BEGAN,
        matches: cantrip_mechanics::triggers::beginning_of_your_combat,
        effect: grow,
        ..TriggeredDef::BLANK
    })],
    ..oracle::OUROBOROID
};
