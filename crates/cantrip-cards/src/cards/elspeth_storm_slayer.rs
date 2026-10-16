use cantrip_api::*;
use cantrip_mechanics::{tokens::SOLDIER, destroy_target};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "If one or more tokens would be created under your control" (CR 111.1, 614.1a): whose they would be, not who makes them.
fn tokens_under_your_control(_: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    matches!(proposal, Proposal::TokenCreation(t) if t.controller == asker.controller && t.count > 0)
}

/// "... twice that many of those tokens are created instead."
fn twice_that_many(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    if let Proposal::TokenCreation(t) = proposal {
        t.count = t.count.saturating_mul(2);
    }
}

/// +1: Create a 1/1 white Soldier creature token.
fn plus_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::tokens::create(g, item.controller, &SOLDIER, 1)?;
    Ok(())
}

/// 0: Put a +1/+1 counter on each creature you control. Those creatures gain flying until your next turn.
fn zero(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creatures = cantrip_mechanics::effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 1);
    for &id in &creatures {
        g.add_effect(
            item.object,
            id,
            EffectKind::AddKeywords(Keywords::of(&[Keyword::Flying])),
            Duration::UntilNextTurnOf(item.controller),
        );
    }
    Ok(())
}

/// "Target creature an opponent controls with mana value 3 or greater" (CR 115.1).
fn opponents_creature_with_value_three_or_more(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.controller(id) != asker.controller && q.mana_value(id) >= 3)
}

/// -3: Destroy target creature an opponent controls with mana value 3 or greater.
fn minus_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    replacements: &[ReplacementDef {
        hint: "twice that many tokens are created",
        kind: ReplacementKind::TokenCreation,
        applies: tokens_under_your_control,
        apply: twice_that_many,
        ..ReplacementDef::BLANK
    }],
    abilities: &[
        ability(loyalty(1), plus_one, "+1: Create a 1/1 white Soldier creature token."),
        ability(loyalty(0), zero, "0: Put a +1/+1 counter on each creature you control. Those creatures gain flying until your next turn."),
        ability(loyalty(-3), minus_three, "\u{2212}3: Destroy target creature an opponent controls with mana value 3 or greater.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: opponents_creature_with_value_three_or_more }]),
    ],
    ..oracle::ELSPETH_STORM_SLAYER
};

pub static EXTRA: &[&CardDef] = &[&SOLDIER];
