use cantrip_api::*;
use cantrip_mechanics::{airbend, triggers::this_enters, targets};
use cantrip_oracle as oracle;

/// "Up to one other target creature or spell": a creature on the battlefield, or a spell on the stack, other than Aang.
fn other_creature_or_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    match t {
        Target::Object(id) if id != asker.source => {
            (targets::permanent(q, id) && q.chars(id).is(CardType::Creature))
                || q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id)
        }
        _ => false,
    }
}

fn airbend_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    airbend(g, item);
    Ok(())
}

/// "Put a +1/+1 counter on each tapped creature you control."
fn counters_on_tapped_creatures(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let tapped: Vec<ObjectId> = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| g.obj(id).tapped && g.chars(id).is(CardType::Creature))
        .collect();
    for id in tapped {
        g.add_counters(id, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever Aang and La attack, put a +1/+1 counter on each tapped creature you control.",
        listens: EventMask::ATTACKS,
        matches: cantrip_mechanics::triggers::this_attacks,
        effect: counters_on_tapped_creatures,
        ..TriggeredDef::BLANK
    })],
    ..oracle::AANG_AND_LA_OCEANS_FURY
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Aang enters, airbend up to one other target creature or spell.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            targets: &[TargetSpec { min: 0, max: 1, filter: other_creature_or_spell }],
            effect: airbend_it,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "Waterbend {8}: Transform Aang. (While paying a waterbend cost, you can tap your artifacts and creatures to help. Each one pays for {1}.)",
            cost: Cost { mana: mana(8, 0, 0, 0, 0, 0), ..Cost::NONE },
            waterbend: true,
            effect: cantrip_mechanics::effects::transform_self,
            ..ActivatedDef::BLANK
        }),
    ],
    back: Some(&BACK),
    ..oracle::AANG_SWIFT_SAVIOR
};
