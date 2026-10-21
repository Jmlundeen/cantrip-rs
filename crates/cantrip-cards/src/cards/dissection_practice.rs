use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target opponent loses 1 life and you gain 1 life. Up to one target creature gets +1/+1 until end of turn. Up to one
/// target creature gets -1/-1 until end of turn." Three target slots (CR 115.1, 601.2c), the creatures' "up to one" each
/// chosen or not and free to be the same creature. If some targets are illegal on resolution the spell still resolves and
/// does what it can, but "does not perform any instructions that would affect an illegal target" (CR 608.2b): "loses 1 life"
/// is done to the opponent and so only while that target is legal, whereas "you gain 1 life" is a separate instruction with
/// no "that much" tying it to the loss and does not affect the opponent at all, so the controller gains the life whether or
/// not the opponent is still a legal target. The creatures are pumped or shrunk each only if its own target is legal. The
/// spell is countered on resolution only when every target is illegal (CR 608.2b), which the core judges.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(opponent) = g.state().spec_targets(item, 0).next().and_then(|slot| targets::player(g, item, slot)) {
        g.lose_life(opponent, 1);
    }
    g.gain_life(item.controller, 1);
    if let Some(creature) = g.state().spec_targets(item, 1).next().and_then(|slot| targets::object(g, item, slot)) {
        pump(g, item.object, creature, 1, 1);
    }
    // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
    g.end_sentence();
    if let Some(creature) = g.state().spec_targets(item, 2).next().and_then(|slot| targets::object(g, item, slot)) {
        pump(g, item.object, creature, -1, -1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[
            targets::OPPONENT.labelled("Opponent to lose 1 life while you gain 1 life"),
            up_to_one(targets::CREATURE.filter).labelled("Creature to get +1/+1 until end of turn"),
            up_to_one(targets::CREATURE.filter).labelled("Creature to get -1/-1 until end of turn"),
        ],
        effect,
    ),
    ..oracle::DISSECTION_PRACTICE
};
