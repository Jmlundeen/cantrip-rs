use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

const EXILED: Question =
    Question::new("Countered", "pay {cost}, or {subject} is countered and exiled?", "", "Pay {cost}", "Let it be countered");

/// Counter target spell unless its controller pays {3} (CR 118.12). If it is countered this way, exile it instead of
/// putting it into its owner's graveyard.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    let payer = g.controller(spell);
    if !g.may_pay(payer, item.object, spell, cost("{3}"), &EXILED)? {
        g.counter_to(spell, CounterTo::Exile);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effect),
    ..oracle::NO_MORE_LIES
};
