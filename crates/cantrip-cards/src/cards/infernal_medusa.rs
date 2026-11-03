use cantrip_api::*;
use cantrip_mechanics::delayed;
use cantrip_oracle as oracle;

/// "Whenever this creature blocks a creature" (CR 509.1, 603.2): about the creature it blocks.
fn blocks_a_creature(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let Event::Blocks { blocker, attacker, .. } = event else { return None };
    if *blocker == asker.source && q.chars(*attacker).is(CardType::Creature) {
        Fired::about(Target::Object(*attacker))
    } else {
        None
    }
}

/// "Whenever this creature becomes blocked by a non-Wall creature" (CR 509.1, 603.2): about the creature that blocks it.
fn becomes_blocked_by_non_wall(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let Event::Blocks { blocker, attacker, .. } = event else { return None };
    let wall = q.words(asker.source).subtype(Subtype::Wall);
    if *attacker == asker.source && q.chars(*blocker).is(CardType::Creature) && !q.chars(*blocker).has_subtype(wall) {
        Fired::about(Target::Object(*blocker))
    } else {
        None
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::BLOCKS, blocks_a_creature),
            delayed::destroy_that_creature_at_end_of_combat,
            "Whenever this creature blocks a creature, destroy that creature at end of combat.",
        ),
        triggered(
            when_asked(EventMask::BLOCKS, becomes_blocked_by_non_wall),
            delayed::destroy_that_creature_at_end_of_combat,
            "Whenever this creature becomes blocked by a non-Wall creature, destroy that creature at end of combat.",
        ),
    ],
    delayed: &[delayed::DESTROY_THAT_CREATURE_AT_END_OF_COMBAT],
    ..oracle::INFERNAL_MEDUSA
};
