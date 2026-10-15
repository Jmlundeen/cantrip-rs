use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_mechanics::{fading, targets};
use cantrip_oracle as oracle;

/// "Exile target creature", remembered as exiled with this enchantment (CR 607.2a): it stays there when this leaves.
fn exile_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.exile_with(&[creature], item.object, false);
    }
    Ok(())
}

/// "Each player returns to the battlefield all cards they own exiled with it": together, each under its owner's control.
fn return_exiled(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<(ObjectId, PlayerId)> =
        g.exiled_with(item.object).into_iter().map(|card| (card, g.obj(card).owner)).collect();
    if !cards.is_empty() {
        g.put_all_onto_battlefield(&cards)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(fading::enters::<5>),
    abilities: &[
        AbilityDef::Activated(ActivatedDef {
            text: "Remove a fade counter from Parallax Wave: Exile target creature.",
            cost: Cost { remove_counters: Some((CounterKind::Fade, 1)), ..Cost::NONE },
            targets: &[targets::CREATURE],
            effect: exile_creature,
            ..ActivatedDef::BLANK
        }),
        fading::upkeep("At the beginning of your upkeep, remove a fade counter from Parallax Wave. If you can't, sacrifice it."),
        AbilityDef::Triggered(TriggeredDef {
            text: "When Parallax Wave leaves the battlefield, each player returns to the battlefield all cards they own exiled with it.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_leaves,
            effect: return_exiled,
            look_back: true,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::PARALLAX_WAVE
};
