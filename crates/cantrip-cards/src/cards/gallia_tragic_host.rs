use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_oracle as oracle;

/// "{4}{B}, Exile another creature card from your graveyard: Return this card from your graveyard to the
/// battlefield tapped with a +1/+1 counter on her" Exiling the other card is a cost; return this card as a new
/// object.
fn return_tapped_with_a_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let owner = g.obj(item.object).owner;
    g.put_onto_battlefield_with_counters(item.object, owner, true, &[(CounterKind::PlusOne, 1)])?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{4}{B}").and(exile_from_graveyard_of(1, filters::creature)),
        return_tapped_with_a_counter,
        "{4}{B}, Exile another creature card from your graveyard: Return this card from your graveyard to the battlefield tapped with a +1/+1 counter on her.",
    )
    .from(ZoneKind::Graveyard)],
    ..oracle::GALLIA_TRAGIC_HOST
};
