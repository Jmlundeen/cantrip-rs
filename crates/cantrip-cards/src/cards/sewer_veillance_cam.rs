use cantrip_api::*;
use cantrip_mechanics::{choose, effects};
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "You may tap or untap target creature": the target is chosen as the trigger goes on the stack; as it resolves its
/// controller chooses Tap, Untap or Do nothing (CR 603.3d, 608.2d, 701.26).
fn tap_or_untap(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let card = g.obj(item.object).card;
    match choose::one_of(g, item.controller, item.object, vec![Label::Card(card, 0), Label::Card(card, 1), Label::Card(card, 2)])? {
        0 => g.tap(creature),
        1 => g.untap(creature),
        _ => {}
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    labels: &["Tap", "Untap", "Do nothing"],
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Sewer-veillance Cam enters, you may tap or untap target creature.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            targets: &[targets::CREATURE],
            effect: tap_or_untap,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When Sewer-veillance Cam leaves the battlefield, you may tap or untap target creature.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_leaves,
            targets: &[targets::CREATURE],
            effect: tap_or_untap,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{3}{U}, Sacrifice Sewer-veillance Cam: Draw two cards.",
            cost: Cost { mana: mana(3, 0, 1, 0, 0, 0), sacrifice_self: true, ..Cost::NONE },
            effect: effects::draw::<2>,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::SEWER_VEILLANCE_CAM
};
