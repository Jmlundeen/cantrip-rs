use cantrip_api::*;
use cantrip_oracle as oracle;

/// Destroy all artifacts and creatures with mana value X or less, X being the life paid as an additional cost.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = item.x as u32;
    let doomed: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let c = g.chars(id);
            (c.is(CardType::Artifact) || c.is(CardType::Creature)) && g.mana_value(id) <= x
        })
        .collect();
    g.destroy_all(&doomed);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // As an additional cost to cast this spell, pay X life.
    additional_cost: Some(Cost { pay_x_life: true, ..Cost::NONE }),
    ..oracle::VICIOUS_RIVALRY
};
