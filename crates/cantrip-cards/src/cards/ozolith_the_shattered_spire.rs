use cantrip_api::*;
use cantrip_mechanics::replacements::{plus_one_counters_applies, plus_one_counters_apply};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::cycling;
use cantrip_oracle as oracle;

/// "If one or more +1/+1 counters would be put on an artifact or creature you control, that many plus one +1/+1
/// counters are put on it instead." (CR 122.6, 614.1a.)
fn applies(q: &dyn Query, asker: &Asker, proposal: &Proposal) -> bool {
    plus_one_counters_applies(q, asker, proposal, &[CardType::Artifact, CardType::Creature])
}

fn apply(_: &dyn Query, _: &Asker, proposal: &mut Proposal) {
    plus_one_counters_apply(proposal);
}

fn artifact_or_creature_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if targets::permanent(q, id) && q.controller(id) == asker.controller && {
        let c = q.chars(id);
        c.is(CardType::Artifact) || c.is(CardType::Creature)
    })
}

fn put_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(Target::Object(permanent)) = item.targets().first().copied() {
        g.add_counters(permanent, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{G}, {T}: Put a +1/+1 counter on target artifact or creature you control. Activate only as a sorcery.",
            cost: Cost { mana: mana(1, 0, 0, 0, 0, 1), tap: true, ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[TargetSpec { min: 1, max: 1, filter: artifact_or_creature_you_control }],
            effect: put_counter,
            ..ActivatedDef::BLANK
        }),
        cycling(mana(2, 0, 0, 0, 0, 0), "Cycling {2} ({2}, Discard this card: Draw a card.)"),
    ],
    replacements: &[ReplacementDef {
        hint: "puts one more +1/+1 counter",
        kind: ReplacementKind::Counters,
        applies,
        apply,
        ..ReplacementDef::BLANK
    }],
    ..oracle::OZOLITH_THE_SHATTERED_SPIRE
};
