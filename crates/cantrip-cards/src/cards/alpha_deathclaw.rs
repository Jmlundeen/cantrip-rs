use cantrip_api::*;
use cantrip_mechanics::{triggers, destroy_target, targets};
use cantrip_oracle as oracle;

fn destroy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

fn becomes_monstrous(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Monstrous { permanent } if *permanent == source => Fired::PLAIN,
        _ => None,
    }
}

fn monstrosity_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.monstrosity(item.object, 4);
    Ok(())
}

const TEXT: &str = "When this creature enters or becomes monstrous, destroy target permanent.";

pub static CARD: CardDef = CardDef {
    abilities: &[
        // "Enters or becomes monstrous" is two triggers with the same effect.
        AbilityDef::Triggered(TriggeredDef {
            text: TEXT,
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            targets: &[targets::PERMANENT],
            effect: destroy_it,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: TEXT,
            listens: EventMask::MONSTROUS,
            matches: becomes_monstrous,
            targets: &[targets::PERMANENT],
            effect: destroy_it,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{5}{B}{G}: Monstrosity 4. (If this creature isn't monstrous, put four +1/+1 counters on it and it becomes monstrous.)",
            cost: Cost::mana(mana(5, 0, 0, 1, 0, 1)),
            effect: monstrosity_four,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::ALPHA_DEATHCLAW
};
