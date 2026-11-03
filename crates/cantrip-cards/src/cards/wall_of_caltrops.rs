use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// An attacker that has left the battlefield or combat has no blockers (CR 506.4), whatever its last-known information says.
/// "If at least one other Wall creature is blocking that creature and no non-Wall creatures are blocking that creature" (CR 509.1h,
/// 603.4): read from the creatures blocking `attacker`, at trigger time and again as the ability resolves. "Wall" is read through the
/// words of this creature's text (CR 612).
fn only_walls_block(q: &dyn Query, source: ObjectId, attacker: ObjectId) -> bool {
    if !q.is_attacking(attacker) {
        return false;
    }
    let wall = q.words(source).subtype(Subtype::Wall);
    let blockers: Vec<ObjectId> = q.creatures_blocking(attacker).into_iter().filter(|&b| q.chars(b).is(CardType::Creature)).collect();
    blockers.iter().any(|&b| b != source && q.chars(b).has_subtype(wall)) && blockers.iter().all(|&b| q.chars(b).has_subtype(wall))
}

/// "Whenever this creature blocks a creature, if ..." (CR 603.2, 603.4): about the creature it blocks.
fn when_creature_blocks_creature(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let fired = triggers::this_blocks_a_creature(q, asker, event)?;
    match fired.subject {
        Target::Object(attacker) if only_walls_block(q, asker.source, attacker) => Some(fired),
        _ => None,
    }
}

/// "This creature gains banding until end of turn", if the condition still holds as it resolves (CR 603.4).
fn if_least_one_other_wall(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(attacker) = item.subject else { return Ok(()) };
    if !only_walls_block(g, item.object, attacker) {
        return Ok(());
    }
    instruction(g, |g| effects::gain_until_end_of_turn(g, item.object, item.object, Keyword::Banding));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::BLOCKS, when_creature_blocks_creature),
            if_least_one_other_wall,
            "Whenever this creature blocks a creature, if at least one other Wall creature is blocking that creature and no non-Wall creatures are blocking that creature, this creature gains banding until end of turn.",
        ),
    ],
    ..oracle::WALL_OF_CALTROPS
};
