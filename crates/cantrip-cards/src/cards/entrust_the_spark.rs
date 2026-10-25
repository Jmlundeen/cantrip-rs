use cantrip_api::*;
use cantrip_mechanics::{effects, library, select};
use cantrip_oracle as oracle;

/// "A planeswalker" (CR 701.21a): the chosen permanent must be a planeswalker.
fn planeswalker(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |card| q.chars(card).is(CardType::Planeswalker))
}

/// "You may sacrifice a planeswalker. If you do, search your library for a planeswalker card, put it onto the battlefield,
/// then shuffle." The sacrifice is chosen as the spell resolves (CR 608.2d) and may be declined; the search follows only a
/// sacrifice that happened (CR 608.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = Selection::new(select::up_to(1)).with(select::part(&(planeswalker as SelectFn), 0, "")).called("planeswalker");
    if !effects::may_sacrifice(g, item.controller, item.object, rule)?.is_empty() {
        library::search_to_battlefield(g, item.controller, item.object, library::Find::ONE, false, |q, id| q.chars(id).is(CardType::Planeswalker))?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::ENTRUST_THE_SPARK };
