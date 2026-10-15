use cantrip_api::*;
use cantrip_mechanics::{exile_target_until_leaves, targets};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// "Target nonland permanent an opponent controls with mana value 2 or less."
fn nonland_opponent_permanent_mv2(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::NONLAND_PERMANENT_AN_OPPONENT_CONTROLS.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.mana_value(id) <= 2)
}

/// Exile it until this artifact leaves the battlefield (CR 610.3): it comes back as soon as the Hole is gone, and is
/// not exiled at all if the Hole already is.
fn exile_until_it_leaves(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target_until_leaves(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this artifact enters, exile target nonland permanent an opponent controls with mana value 2 or less until this artifact leaves the battlefield.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[TargetSpec { min: 1, max: 1, filter: nonland_opponent_permanent_mv2 }],
        effect: exile_until_it_leaves,
        ..TriggeredDef::BLANK
    })],
    ..oracle::PORTABLE_HOLE
};
