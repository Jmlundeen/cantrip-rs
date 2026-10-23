use cantrip_api::*;
use cantrip_mechanics::{targets, tokens, triggers};
use cantrip_oracle as oracle;

/// A 1/1 colorless Construct artifact creature token.
pub static CONSTRUCT: CardDef = CardDef {
    types: TypeSet::of(&[CardType::Artifact, CardType::Creature]),
    power: Some(1),
    toughness: Some(1),
    ..token!(Construct)
};

/// "Whenever one or more artifacts you control leave the battlefield during your turn" (CR 603.6c, 603.10a, 603.2c): an
/// artifact as it last was on the battlefield (this one too), under its controller's control, on its controller's turn.
fn artifact_you_control_leaves_on_your_turn(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let left = triggers::permanent_you_control_left_on_your_turn(q, me, event)?;
    if q.chars(left).is(CardType::Artifact) { Fired::PLAIN } else { None }
}

/// "Create a 1/1 colorless Construct artifact creature token." (CR 111.1.)
fn create_construct(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &CONSTRUCT, 1)?;
    Ok(())
}

/// "This artifact deals 1 damage to each opponent. You gain 1 life." Two sentences, so two instructions (CR 608.2c).
fn ping_and_gain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::damage_each_opponent(g, item, 1);
    g.end_sentence();
    cantrip_mechanics::effects::gain_one_life(g, item)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::ZONE_CHANGE, artifact_you_control_leaves_on_your_turn),
            create_construct,
            "Whenever one or more artifacts you control leave the battlefield during your turn, create a 1/1 colorless Construct artifact creature token. This ability triggers only once each turn.",
        )
        .once_per_batch()
        .triggers_each_turn(1),
        ability(
            TAP.and(sacrifice(1, targets::filters::artifact)),
            ping_and_gain,
            "{T}, Sacrifice an artifact: This artifact deals 1 damage to each opponent. You gain 1 life.",
        ),
    ],
    ..oracle::ONI_CULT_ANVIL
};

pub static EXTRA: &[&CardDef] = &[&CONSTRUCT];
