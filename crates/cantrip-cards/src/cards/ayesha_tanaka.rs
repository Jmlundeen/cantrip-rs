use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "{T}: Counter target activated ability from an artifact source unless that ability's controller pays {W}." (CR 608.2, 701.6a, 118.1).
fn counter_target_activated_ability_artifact(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let Target::Ability(serial) = g.state().target(item, 0) else { return Ok(()) };
    let payer = g.state().stack.iter().find(|entry| entry.serial == serial).map(|entry| entry.controller);
    if let Some(payer) = payer
        && !g.may_pay(payer, item.object, item.object, cost("{W}"), &PAY_QUESTION)?
    {
        // The target was checked at the start of resolution (CR 608.2b). Mana abilities used
        // during payment can change its source's types, but do not cause another legality check.
        if let Some(at) = g.state().stack.iter().position(|entry| entry.serial == serial) {
            let source = g.state().stack[at].object;
            g.counter_item(source, at);
        }
    }
    Ok(())
}

static PAY_QUESTION: Question = Question::new(
    "Pay",
    "pay {cost}?",
    "",
    "Pay {cost}",
    "Decline",
);

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                TAP,
                counter_target_activated_ability_artifact,
                "{T}: Counter target activated ability from an artifact source unless that ability's controller pays {W}.",
            )
            .targets(&[target(targets::activated_ability_from_an_artifact_source)]),
    ],
    ..oracle::AYESHA_TANAKA
};
