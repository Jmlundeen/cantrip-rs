use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::library::{search_to_graveyard, Find};
use cantrip_mechanics::targets::card_in_graveyard;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// When Goblin Engineer enters, you may search your library for an artifact card, put it into your graveyard, then
/// shuffle. Declining skips the search and the shuffle.
fn search_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if !cantrip_mechanics::effects::ask_may(g, player, item.object, Purpose::Optional)? {
        return Ok(());
    }
    search_to_graveyard(g, player, item.object, Find::ONE, |q, c| q.chars(c).is(CardType::Artifact))?;
    Ok(())
}

/// "Target artifact card with mana value 3 or less from your graveyard".
fn cheap_artifact_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    card_in_graveyard(q, t, &[CardType::Artifact])
        && matches!(t, Target::Object(id) if q.obj(id).owner == asker.controller && q.mana_value(id) <= 3)
}

/// Return target artifact card from your graveyard to the battlefield.
fn reanimate_artifact(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.put_onto_battlefield(card, item.controller, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, you may search your library for an artifact card, put it into your graveyard, then shuffle.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: search_artifact,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{R}, {T}, Sacrifice an artifact: Return target artifact card with mana value 3 or less from your graveyard to the battlefield.",
            cost: Cost {
                mana: mana(0, 0, 0, 0, 1, 0),
                tap: true,
                permanents: Some(PermanentCost { count: 1, filter: targets::filters::artifact, to_hand: false }),
                ..Cost::NONE
            },
            targets: &[TargetSpec { min: 1, max: 1, filter: cheap_artifact_in_your_graveyard }],
            effect: reanimate_artifact,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::GOBLIN_ENGINEER
};
