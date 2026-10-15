use cantrip_api::*;
use cantrip_mechanics::retrace;
use cantrip_mechanics::{damage_target, targets, tokens};
use cantrip_oracle as oracle;

/// "Target land card from your graveyard" (CR 115.1).
fn land_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[CardType::Land])
}

/// +1: Return up to one target land card from your graveyard to your hand.
fn return_land(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(land) = targets::object(g, item, 0) {
        g.return_to_hand(&[land]);
    }
    Ok(())
}

/// -1: Wrenn and Six deals 1 damage to any target.
fn ping(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 1);
    Ok(())
}

/// -7: You get an emblem.
fn emblem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_emblem(g, item.controller, &EMBLEM);
    Ok(())
}

// --- The emblem ---------------------------------------------------------------------------------------------------

/// Instant and sorcery cards in the emblem's controller's graveyard (CR 702.81a): a card that is only a creature there,
/// as an Adventurer is (CR 715.4), is not one.
fn instant_or_sorcery_in_your_graveyard(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    q.state().obj(affected).zone == Zone::graveyard(q.state().obj(source).controller)
        && (so_far.is(CardType::Instant) || so_far.is(CardType::Sorcery))
}

/// "Instant and sorcery cards in your graveyard have retrace."
pub static EMBLEM: CardDef = CardDef {
    name: "Wrenn and Six emblem",
    has_cost: false,
    statics: &[StaticDef {
        text: "Instant and sorcery cards in your graveyard have retrace.",
        layer: Layer::Ability,
        affects: instant_or_sorcery_in_your_graveyard,
        apply: retrace::give,
        everywhere: true,
        ..StaticDef::BLANK
    }],
    text: "Instant and sorcery cards in your graveyard have retrace.",
    ..CardDef::BLANK
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "+1: Return up to one target land card from your graveyard to your hand.",
            cost: Cost { loyalty: Some(1), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[TargetSpec { min: 0, max: 1, filter: land_in_your_graveyard }],
            effect: return_land,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}1: Wrenn and Six deals 1 damage to any target.",
            cost: Cost { loyalty: Some(-1), ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[targets::ANY],
            effect: ping,
            ..ActivatedDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "\u{2212}7: You get an emblem with \"Instant and sorcery cards in your graveyard have retrace.\"",
            cost: Cost { loyalty: Some(-7), ..Cost::NONE },
            timing: Timing::Sorcery,
            effect: emblem,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::WRENN_AND_SIX
};

pub static EXTRA: &[&CardDef] = &[&EMBLEM];
