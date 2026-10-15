use cantrip_api::*;
use cantrip_mechanics::{kicker, targets};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// He deals damage equal to his power to each of up to X targets, X being the number of times he was kicked: the
/// targets are chosen as the trigger goes on the stack, counted by the kicks (`TargetSpec::COUNTED`).
fn damage_each(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = g.chars(item.object).power.unwrap_or(0).max(0) as u16;
    let hits: Vec<(ObjectId, Target, u16)> =
        (0..item.target_count as usize).filter(|&slot| g.target_legal(item, slot)).map(|slot| (item.object, item.targets[slot], power)).collect();
    g.damage_together(&hits);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    kicker: Some(Kicker { cost: mana(2, 0, 0, 0, 0, 0), multi: true, waterbend: false }),
    enters: Some(kicker::enters_with_counters::<{ CounterKind::PlusOne as usize }>),
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When Batroc enters, he deals damage equal to his power to each of up to X targets, where X is the number of times he was kicked.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[TargetSpec { min: 0, max: TargetSpec::COUNTED, filter: targets::ANY.filter }],
        effect: damage_each,
        ..TriggeredDef::BLANK
    })],
    ..oracle::BATROC_THE_LEAPER
};
