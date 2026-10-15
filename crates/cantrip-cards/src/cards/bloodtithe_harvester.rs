use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::this_enters;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

fn create_blood(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_blood(g, item.controller)?;
    Ok(())
}

/// The target creature gets -X/-X until end of turn, X twice the Blood tokens its controller controls as this
/// resolves.
fn shrink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let blood = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&p| g.obj(p).token && g.chars(p).has_subtype(Subtype::Blood))
        .count() as i16;
    pump(g, item.object, creature, -2 * blood, -2 * blood);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Bloodtithe Harvester enters, create a Blood token.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: create_blood,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}, Sacrifice Bloodtithe Harvester: Target creature gets -X/-X until end of turn, where X is twice the number of Blood tokens you control. Activate only as a sorcery.",
            cost: Cost { tap: true, sacrifice_self: true, ..Cost::NONE },
            timing: Timing::Sorcery,
            targets: &[targets::CREATURE],
            effect: shrink,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::BLOODTITHE_HARVESTER
};

pub static EXTRA: &[&CardDef] = &[&tokens::BLOOD];
