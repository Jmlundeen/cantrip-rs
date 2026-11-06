use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Exile target nonland permanent. You lose 3 life." (CR 115.1).
fn exile_target_nonland_permanent_lose_target_0(
    q: &dyn Query,
    _: &Asker,
    t: Target,
) -> bool {
    matches!(
        t, Target::Object(id) if cantrip_mechanics::targets::permanent(q, id) && (! (q
        .chars(id).is(CardType::Land)))
    )
}

/// "Exile target nonland permanent. You lose 3 life." (CR 608.2, 701.13a, 119.3).
fn exile_target_nonland_permanent_lose(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    let t_0 = vec![g.state().target(item, 0)];
    let objects_0 = t_0
        .get(0)
        .copied()
        .and_then(|t: Target| t.object())
        .into_iter()
        .collect::<Vec<_>>();
    g.exile_all(&objects_0);
    g.end_sentence();
    g.lose_life(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[target(exile_target_nonland_permanent_lose_target_0)],
        exile_target_nonland_permanent_lose,
    ),
    ..oracle::ANGUISHED_UNMAKING
};
