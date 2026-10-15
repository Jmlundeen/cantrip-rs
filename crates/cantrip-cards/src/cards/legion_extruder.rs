use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::this_enters;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

fn deal_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, 2);
    Ok(())
}

fn another_artifact(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    id != asker.source && q.chars(id).is(CardType::Artifact) && q.controller(id) == asker.controller
}

fn golem(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::GOLEM, 1)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this artifact enters, it deals 2 damage to any target.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[targets::ANY],
            effect: deal_two,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{2}, {T}, Sacrifice another artifact: Create a 3/3 colorless Golem artifact creature token.",
            cost: Cost {
                mana: mana(2, 0, 0, 0, 0, 0),
                tap: true,
                permanents: Some(PermanentCost { count: 1, filter: another_artifact, to_hand: false }),
                ..Cost::NONE
            },
            effect: golem,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::LEGION_EXTRUDER
};

pub static EXTRA: &[&CardDef] = &[&tokens::GOLEM];
