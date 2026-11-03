use cantrip_api::*;
use cantrip_mechanics::{counters, effects, targets};
use cantrip_oracle as oracle;

/// "Tap up to X target creatures, then distribute three stun counters among any number of tapped creatures your opponents control."
/// The targets are tapped (CR 701.26a, 608.2b: those still legal), and that sentence is over before the next reads the board
/// (CR 608.2c, "then"), so a creature already tapped, or one of an opponent's that was tapped before, is among those the counters may go
/// on. They are not targeted: their controller decides where each goes as the spell resolves (CR 608.2d), from the creatures that
/// are tapped and an opponent's then.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::tap_targets(g, item)?;
    g.end_sentence();
    let tapped: Vec<ObjectId> = g
        .state()
        .opponents(item.controller)
        .flat_map(|opponent| g.permanents_of(opponent))
        .filter(|&id| g.chars(id).is(CardType::Creature) && g.state().obj(id).tapped)
        .collect();
    counters::distribute(g, item.controller, item.object, &tapped, CounterKind::Stun, 3)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "up to X target creatures": X is announced with the spell, along with its waterbend cost (CR 107.3a, 601.2b).
    spell: spell(&[targets::CREATURE.between(0, TargetSpec::COUNTED)], effect),
    // "As an additional cost to cast this spell, waterbend {X}." (CR 701.67a-b, 601.2f)
    additional_cost: Some(Cost::waterbend(0, 1)),
    ..oracle::CRASHING_WAVE
};
