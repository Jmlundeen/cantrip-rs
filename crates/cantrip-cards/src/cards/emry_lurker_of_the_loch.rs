use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::affinity;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn mill_four(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.mill(item.controller, 4);
    Ok(())
}

fn artifact_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    cantrip_mechanics::targets::card_in_your_graveyard(q, asker, t, &[CardType::Artifact])
}

/// {T}: choose target artifact card in your graveyard. You may cast that card this turn: it stays in the graveyard, and
/// costs and timing are as usual (CR 601.3).
fn allow_cast(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(card) = targets::object(g, item, 0) {
        g.allow_play(card, item.controller, PlayGrant { expiry: Expiry::EndOfTurn, ..PlayGrant::CAST });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[affinity::for_artifacts],
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Emry, Lurker of the Loch enters, mill four cards.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: mill_four,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{T}: Choose target artifact card in your graveyard. You may cast that card this turn.",
            cost: Cost { tap: true, ..Cost::NONE },
            targets: &[TargetSpec { min: 1, max: 1, filter: artifact_card_in_your_graveyard }],
            effect: allow_cast,
            ..ActivatedDef::BLANK
        }),
    ],
    ..oracle::EMRY_LURKER_OF_THE_LOCH
};
