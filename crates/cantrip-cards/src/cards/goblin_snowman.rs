use cantrip_api::*;
use cantrip_mechanics::{effects, prevention, targets, triggers};
use cantrip_oracle as oracle;

fn prevent_combat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = item.subject.object() {
        g.add_effect(item.object, creature, EffectKind::Prevention { card: item.ability.card, index: 0, to: Target::None, shield: Shield::All }, Duration::EndOfTurn);
    }
    Ok(())
}

/// A creature this source is currently blocking (CR 506.4).
fn creature_blocked_by_this(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.chars(id).is(CardType::Creature)
        && q.state().blocks.iter().any(|b| b.attacker == id && b.blocker == asker.source))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_BLOCKS, prevent_combat, "Whenever this creature blocks, prevent all combat damage that would be dealt to and dealt by it this turn."),
        ability(TAP, effects::damage_first_target::<1>, "{T}: This creature deals 1 damage to target creature it's blocking.")
            .targets(&[target(creature_blocked_by_this)]),
    ],
    granted_preventions: &[prevention::COMBAT_DAMAGE_DEALT_TO_AND_BY.text("Whenever this creature blocks, prevent all combat damage that would be dealt to and dealt by it this turn.")],
    ..oracle::GOBLIN_SNOWMAN
};
