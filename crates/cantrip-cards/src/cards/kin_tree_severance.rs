use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Exile target permanent with mana value 3 or greater."
fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(targets::permanent_with_mana_value_at_least::<3>)], exile),
    ..oracle::KIN_TREE_SEVERANCE
};
