use cantrip_api::*;
use cantrip_mechanics::{choose, targets};
use cantrip_oracle as oracle;

/// "When this creature transforms into Revealing Eye": the ability is the back face's, so it is there to see only
/// once the permanent has turned over.
fn this_transforms(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Transformed { object } if *object == source => Fired::PLAIN,
        _ => None,
    }
}

/// "Target opponent reveals their hand. You may choose a nonland card from it. If you do, that player discards that
/// card, then draws a card." The whole hand is revealed and the card is chosen by the controller, up to one, among the
/// nonland cards; choosing none ends the ability.
fn reveal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Player(opponent) = item.targets[0] else { return Ok(()) };
    let hand = g.state().zone(Zone::hand(opponent)).to_vec();
    if hand.is_empty() {
        return Ok(());
    }
    g.reveal(opponent, &hand);
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| !g.chars(c).is(CardType::Land)).collect();
    let Some(&card) = choose::cards_from(g, item.controller, item.object, Purpose::Discard, hand, eligible, 1, true)?.first() else {
        return Ok(());
    };
    g.discard(card);
    g.draw(opponent, 1);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this creature transforms into Revealing Eye, target opponent reveals their hand. You may choose a nonland card from it. If you do, that player discards that card, then draws a card.",
        listens: EventMask::TRANSFORMED,
        matches: this_transforms,
        targets: &[targets::OPPONENT],
        effect: reveal,
        ..TriggeredDef::BLANK
    })],
    ..oracle::REVEALING_EYE
};

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{2}{B}: Transform Concealing Curtains. Activate only as a sorcery.",
        cost: Cost { mana: mana(2, 0, 0, 1, 0, 0), ..Cost::NONE },
        timing: Timing::Sorcery,
        effect: cantrip_mechanics::effects::transform_self,
        ..ActivatedDef::BLANK
    })],
    back: Some(&BACK),
    ..oracle::CONCEALING_CURTAINS
};
