use cantrip_api::*;
use cantrip_oracle as oracle;

/// I: "You draw two cards and you lose 2 life."
fn chapter_one(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 2);
    g.lose_life(item.controller, 2);
    Ok(())
}

/// II, III: the delayed ability that lasts the turn (CR 603.7b), made by the chapter. "You" is the Saga's controller as
/// the chapter resolved (CR 603.7d).
fn watch(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let this = g.card_id(&CARD);
    g.delayed_trigger(DelayedWhen::ThisTurn, item.object, item.controller, AbilityRef::delayed(this, 0), Target::None);
    Ok(())
}

/// "Each opponent loses 1 life and you gain 1 life."
fn drain(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.each_opponent_loses_life(item.controller, 1);
    g.gain_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "I \u{2014} You draw two cards and you lose 2 life.",
            chapter: 1,
            effect: chapter_one,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "II \u{2014} Whenever a creature you control enters this turn, each opponent loses 1 life and you gain 1 life.",
            chapter: 2,
            effect: watch,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "III \u{2014} Whenever a creature you control enters this turn, each opponent loses 1 life and you gain 1 life.",
            chapter: 3,
            effect: watch,
            ..TriggeredDef::BLANK
        }),
    ],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever a creature you control enters this turn, each opponent loses 1 life and you gain 1 life.",
        listens: EventMask::ZONE_CHANGE,
        matches: cantrip_mechanics::triggers::creature_you_control_enters,
        effect: drain,
        ..TriggeredDef::BLANK
    })],
    ..oracle::THUNDER_OF_UNITY
};
