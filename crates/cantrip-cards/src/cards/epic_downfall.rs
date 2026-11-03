use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Exile target creature with mana value 3 or greater." (CR 701.13a, 202.3): a creature of either player's, asked as the spell is
/// cast and again as it resolves (CR 608.2b); X in a mana cost is 0 on the battlefield.
fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::exile_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(targets::creature_with_mana_value_at_least::<3>)], exile),
    ..oracle::EPIC_DOWNFALL
};
