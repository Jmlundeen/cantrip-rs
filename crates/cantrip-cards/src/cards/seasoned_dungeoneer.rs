use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::{explore, triggers};
use cantrip_oracle as oracle;

/// "Target attacking Cleric, Rogue, Warrior, or Wizard" (CR 115.1): an attacking creature of one of those types.
fn attacking_party_member(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.is_attacking(id) && {
            let c = q.chars(id);
            let words = q.words(asker.source);
            [Subtype::Cleric, Subtype::Rogue, Subtype::Warrior, Subtype::Wizard].iter().any(|&s| c.has_subtype(words.subtype(s)))
        })
}

/// It gains protection from creatures until end of turn. It explores.
fn protect_and_explore(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_effect(
        item.object,
        creature,
        EffectKind::ProtectionFromTypes(TypeSet::of(&[CardType::Creature])),
        Duration::EndOfTurn,
    );
    explore(g, creature)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, cantrip_mechanics::effects::take_the_initiative, "When this creature enters, you take the initiative."),
        triggered(triggers::YOU_ATTACK, protect_and_explore, "Whenever you attack, target attacking Cleric, Rogue, Warrior, or Wizard gains protection from creatures until end of turn. It explores.")
            .targets(&[target(attacking_party_member)]),
    ],
    ..oracle::SEASONED_DUNGEONEER
};
