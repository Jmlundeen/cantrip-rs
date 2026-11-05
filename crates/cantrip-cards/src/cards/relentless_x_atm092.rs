use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Return this card from your graveyard to the battlefield tapped with a finality counter on it" (CR 400.7, 122.1h): the
/// counter is on it as it enters, so it is exiled instead of dying. Nothing if the card has left the graveyard (CR 400.7).
fn return_tapped_with_finality(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.put_onto_battlefield_with_counters(item.object, item.controller, true, &[(CounterKind::Finality, 1)])?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[Keyword::CantBeBlockedExceptByThreeOrMore]),
    abilities: &[ability(
        cost("{8}"),
        return_tapped_with_finality,
        "{8}: Return this card from your graveyard to the battlefield tapped with a finality counter on it. (If a creature with a finality counter on it would die, exile it instead.)",
    )
    .from(ZoneKind::Graveyard)],
    ..oracle::RELENTLESS_X_ATM092
};
