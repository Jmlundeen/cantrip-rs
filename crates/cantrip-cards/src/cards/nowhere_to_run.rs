use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::this_enters};
use cantrip_oracle as oracle;

/// "Target creature an opponent controls gets -3/-3 until end of turn" (CR 611.2a): -3/-3 from the layer 7c, on the creature
/// if it is still the creature that was targeted.
fn minus_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0)
        && g.is_on_battlefield(creature)
    {
        g.add_effect(item.object, creature, EffectKind::Pump(-3, -3), Duration::EndOfTurn);
    }
    Ok(())
}

/// "Creatures your opponents control": a creature, as the earlier layers made it, under the control of an opponent of the
/// source's controller (CR 102.2). Judged now, so a creature that changes hands, or becomes one, is followed.
fn opponents_creature(q: &dyn LayerQuery, source: ObjectId, subject: ObjectId) -> bool {
    let s = q.state();
    s.obj(subject).controller != s.obj(source).controller && q.chars(subject).is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this enchantment enters, target creature an opponent controls gets -3/-3 until end of turn.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[targets::CREATURE_AN_OPPONENT_CONTROLS],
        effect: minus_three,
        ..TriggeredDef::BLANK
    })],
    targeting_rules: &[
        // CR 702.11b, 609.4: "as though they didn't have hexproof", so hexproof granted later is ignored too; shroud and
        // protection are not touched. Also the rule for Nowhere to Run's own enters trigger, which targets with it.
        TargetingRule { kind: TargetingRuleKind::IgnoreHexproof, applies: opponents_creature },
        // CR 702.21a: "Ward abilities of those creatures don't trigger."
        TargetingRule { kind: TargetingRuleKind::SuppressWard, applies: opponents_creature },
    ],
    ..oracle::NOWHERE_TO_RUN
};
