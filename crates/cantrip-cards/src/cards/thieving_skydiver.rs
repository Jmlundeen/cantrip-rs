use cantrip_api::*;
use cantrip_mechanics::{targets, triggers::this_enters};
use cantrip_oracle as oracle;

/// "Target artifact with mana value X or less": the X it was kicked with, read from the creature (CR 702.33a).
fn artifact_with_value_at_most_x(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let x = q.obj(asker.source).x as u32;
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id) && q.chars(id).is(CardType::Artifact) && q.mana_value(id) <= x)
}

/// "If it was kicked" (CR 603.4).
fn was_kicked(q: &dyn Query, asker: &Asker) -> bool {
    q.obj(asker.source).kicked > 0
}

/// Gain control of the artifact (CR 108.4, no duration); an Equipment is then attached to this creature.
fn take_and_attach(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(artifact) = targets::object(g, item, 0) {
        g.gain_control(item.object, artifact, item.controller, Duration::Indefinite);
        if g.chars(artifact).has_subtype(Subtype::Equipment) && is_live(g, item.object) && g.is_on_battlefield(item.object) {
            g.attach(artifact, item.object);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: Some(Kicker { cost: ManaCost { x: 1, ..ManaCost::ZERO }, multi: false, waterbend: false }),
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this creature enters, if it was kicked, gain control of target artifact with mana value X or less. If that artifact is an Equipment, attach it to this creature.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[TargetSpec { min: 1, max: 1, filter: artifact_with_value_at_most_x }],
        effect: take_and_attach,
        condition: Some(was_kicked),
        ..TriggeredDef::BLANK
    })],
    ..oracle::THIEVING_SKYDIVER
};
