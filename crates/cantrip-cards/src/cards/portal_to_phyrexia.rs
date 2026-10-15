use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::{each_opponent_sacrifices, triggers};
use cantrip_oracle as oracle;

fn creature_card(q: &dyn Query, _: &Asker, t: Target) -> bool {
    targets::card_in_graveyard(q, t, &[CardType::Creature])
}

/// Each opponent sacrifices three creatures of their choice (CR 701.21a), in turn order; a player with three or fewer
/// sacrifices them all without being asked.
fn opponents_sacrifice_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 3, |q, id| q.chars(id).is(CardType::Creature))
}

/// Put target creature card from a graveyard onto the battlefield under your control. It's a Phyrexian in addition
/// to its other types, for as long as it is that object (CR 611.2a).
fn reanimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let entered = g.put_onto_battlefield(card, item.controller, false)?;
    if g.obj(entered).zone.is(ZoneKind::Battlefield) {
        g.add_effect(item.object, entered, EffectKind::AddSubtype(g.words(item.object).subtype(Subtype::Phyrexian)), Duration::Indefinite);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this artifact enters, each opponent sacrifices three creatures of their choice.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            effect: opponents_sacrifice_three,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "At the beginning of your upkeep, put target creature card from a graveyard onto the battlefield under your control. It's a Phyrexian in addition to its other types.",
            listens: EventMask::STEP_BEGAN,
            matches: triggers::your_upkeep,
            targets: &[TargetSpec { min: 1, max: 1, filter: creature_card }],
            effect: reanimate,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::PORTAL_TO_PHYREXIA
};
