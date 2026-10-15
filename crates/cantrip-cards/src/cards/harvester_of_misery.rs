use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// Other creatures get -2/-2 until end of turn: the creatures on the battlefield as the ability resolves, this one not
/// among them.
fn shrink_others(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let others: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| id != item.object && g.chars(id).is(CardType::Creature))
        .collect();
    for id in others {
        pump(g, item.object, id, -2, -2);
    }
    Ok(())
}

/// Target creature gets -2/-2 until end of turn.
fn shrink_target(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, -2, -2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Harvester of Misery enters, other creatures get -2/-2 until end of turn.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: shrink_others,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{1}{B}, Discard Harvester of Misery: Target creature gets -2/-2 until end of turn.",
            cost: Cost { mana: mana(1, 0, 0, 1, 0, 0), discard_self: true, ..Cost::NONE },
            targets: &[targets::CREATURE],
            effect: shrink_target,
            from: ZoneKind::Hand,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::HARVESTER_OF_MISERY
};
