use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "This creature can't be blocked by artifact creatures" (CR 509.1b, 604.1): a blocker that is an artifact creature may
/// not be declared against it. The rule is the source's own, so it is `block_rules`.
fn cant_be_blocked_by_artifact_creatures(q: &dyn Query, source: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    !(attacker == source && q.chars(blocker).is(CardType::Artifact) && q.chars(blocker).is(CardType::Creature))
}

/// The static ability names this creature alone (CR 604.1, 201.5).
fn itself(_: &dyn LayerQuery, source: ObjectId, subject: ObjectId) -> bool {
    source == subject
}

/// "This creature can't be the target of abilities from artifact sources your opponents control" (CR 702.18a by way of
/// CR 115.1, 604.1): a non-spell ability whose source is an artifact, controlled by an opponent of this creature's
/// controller. A spell from an artifact is not an ability (CR 113.3a).
fn cant_be_targeted_by_an_opponents_artifact_ability(q: &dyn LayerQuery, source: ObjectId, by: &Asker, by_source: &Chars) -> bool {
    !by.spell && by_source.is(CardType::Artifact) && by.controller != q.controller(source)
}

pub static CARD: CardDef = CardDef {
    // "Strange new worlds — When this creature enters, you gain life equal to the number of differently named lands you
    // control" (CR 603.6a, 201.2a): counted as the ability resolves, so a land that entered with it counts (CR 608.2c).
    abilities: &[triggered(
        THIS_ENTERS,
        effects::gain_life_for_differently_named_lands,
        "Strange new worlds \u{2014} When this creature enters, you gain life equal to the number of differently named lands you control.",
    )],
    block_rules: &[cant_be_blocked_by_artifact_creatures],
    targeting_rules: &[TargetingRule {
        kind: TargetingRuleKind::Forbid(cant_be_targeted_by_an_opponents_artifact_ability),
        applies: itself,
    }],
    ..oracle::SPECIES_8472
};
