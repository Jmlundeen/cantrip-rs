use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Up to one other target creature."
fn other_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source) && (targets::CREATURE.filter)(q, asker, t)
}

/// "He fights up to one other target creature" (CR 701.12). A Wolverine that left and came back is a new object, not
/// "he" (CR 400.7), and fights nothing: `fight` checks that both are still creatures on the battlefield.
fn fight(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(other) = targets::object(g, item, 0) {
        g.fight(item.object, other);
    }
    Ok(())
}

/// "If damage would be dealt to Wolverine, instead that damage is dealt, but all other damage already dealt to him is
/// healed."
fn damage_to_him(_: &dyn Query, asker: &Asker, p: &Proposal) -> bool {
    matches!(p, Proposal::Damage(d) if d.target == Target::Object(asker.source) && d.amount > 0)
}

fn heal_the_rest(_: &dyn Query, _: &Asker, p: &mut Proposal) {
    if let Proposal::Damage(d) = p {
        d.heal_marked = true;
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, fight, "When Wolverine enters, he fights up to one other target creature.")
        .targets(&[TargetSpec { min: 0, max: 1, filter: other_creature }])],
    replacements: &[ReplacementDef {
        hint: "heal damage already dealt to Wolverine",
        kind: ReplacementKind::Damage,
        applies: damage_to_him,
        apply: heal_the_rest,
        ..ReplacementDef::BLANK
    }],
    ..oracle::WOLVERINE_FIERCE_FIGHTER
};
