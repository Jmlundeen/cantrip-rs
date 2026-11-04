use cantrip_api::*;
use cantrip_mechanics::{select, targets};
use cantrip_oracle as oracle;

/// "Return up to six target creature cards with different names from your graveyard to the battlefield." The targets still
/// legal as it resolves (CR 608.2b) enter together under your control (CR 110.2a, 400.7). The names are judged as the targets are
/// chosen and again on resolution (CR 115.1, 201.2b), by the core.
fn return_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<(ObjectId, PlayerId)> = targets::legal_objects(g, item).into_iter().map(|c| (c, item.controller)).collect();
    g.put_all_onto_battlefield(&cards)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD.between(0, 6).together(select::DIFFERENT_NAMES)],
        return_them,
    ),
    ..oracle::BEHOLD_THE_SINISTER_SIX
};
