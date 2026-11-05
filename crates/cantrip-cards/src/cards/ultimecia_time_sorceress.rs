use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::{THIS_ENTERS_OR_ATTACKS, THIS_TRANSFORMS, YOUR_END_STEP};
use cantrip_oracle as oracle;

const TRANSFORM: Question = Question::new("Transform", "pay {cost} and exile eight cards from your graveyard to transform {subject}?", "", "Pay {cost}", "Don't transform");

/// "At the beginning of your end step, you may pay {4}{U}{U}{B}{B} and exile eight cards from your graveyard. If you do, transform
/// Ultimecia." (CR 118.12, 608.2d, 701.27): one payment made as the ability resolves, the mana and the eight cards together, with
/// fewer than eight cards in the graveyard it can't be paid. It transforms only if it is still the permanent the ability is of and
/// has not transformed since (CR 701.27f).
fn pay_and_exile_to_transform(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cost = Cost { mana: mana("{4}{U}{U}{B}{B}"), exile_from_graveyard: 8, ..Cost::NONE };
    if g.may_pay(item.controller, item.object, item.object, cost, &TRANSFORM)? {
        g.transform_by(item.object, item)?;
    }
    Ok(())
}

/// Time Compression: "When this creature transforms into Ultimecia, Omnipotent, take an extra turn after this one." (CR 500.7)
fn take_an_extra_turn(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_extra_turns(item.controller, 1);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(
        THIS_TRANSFORMS,
        take_an_extra_turn,
        "Time Compression \u{2014} When this creature transforms into Ultimecia, Omnipotent, take an extra turn after this one.",
    )],
    ..oracle::ULTIMECIA_OMNIPOTENT
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS_OR_ATTACKS, |g, item| effects::surveil(g, item, 2), "Whenever Ultimecia enters or attacks, surveil 2."),
        triggered(
            YOUR_END_STEP,
            pay_and_exile_to_transform,
            "At the beginning of your end step, you may pay {4}{U}{U}{B}{B} and exile eight cards from your graveyard. If you do, transform Ultimecia.",
        ),
    ],
    back: Some(&BACK),
    ..oracle::ULTIMECIA_TIME_SORCERESS
};
