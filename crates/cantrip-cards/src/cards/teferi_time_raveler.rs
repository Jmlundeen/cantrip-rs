use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Each opponent can cast spells only any time they could cast a sorcery (CR 307.1, 101.2): the active player in a
/// main phase with the stack empty. The prohibition reaches a spell another effect offers during resolution too.
fn opponents_cast_as_sorcery(q: &dyn Query, asker: &Asker, caster: PlayerId) -> bool {
    let s = q.state();
    caster == asker.controller || (caster == s.turn.active && s.turn.step().is_main() && s.stack.is_empty())
}

/// +1: until your next turn, you may cast sorcery spells as though they had flash (CR 611.2c, a rule that covers cards
/// drawn after this resolves).
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_effect(
        item.object,
        item.object,
        EffectKind::CastAsThoughFlash(TypeSet::of(&[CardType::Sorcery])),
        Duration::UntilNextTurnOf(item.controller),
    );
    Ok(())
}

fn artifact_creature_or_enchantment(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::PERMANENT.filter)(q, asker, t) && matches!(t, Target::Object(id) if {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Creature) || c.is(CardType::Enchantment)
    })
}

/// -3: return up to one target artifact, creature, or enchantment to its owner's hand. Draw a card.
fn minus_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(id) = targets::object(g, item, 0) {
        g.return_to_hand(&[id]);
    }
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), plus_one, "+1: Until your next turn, you may cast sorcery spells as though they had flash."),
        ability(loyalty(-3), minus_three, "\u{2212}3: Return up to one target artifact, creature, or enchantment to its owner's hand. Draw a card.")
            .targets(&[TargetSpec { min: 0, max: 1, filter: artifact_creature_or_enchantment, label: "" }]),
    ],
    cast_rules: &[opponents_cast_as_sorcery],
    ..oracle::TEFERI_TIME_RAVELER
};
