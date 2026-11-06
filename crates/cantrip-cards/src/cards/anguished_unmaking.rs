use cantrip_api::*;
use cantrip_mechanics::{exile_target, targets};
use cantrip_oracle as oracle;

/// "Exile target nonland permanent. You lose 3 life." (CR 608.2, 701.13a, 119.3). The only target is checked on
/// resolution, so a spell whose target is illegal never resolves and its controller loses no life (CR 608.2b).
fn exile_target_nonland_permanent_lose(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    g.lose_life(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT], exile_target_nonland_permanent_lose),
    ..oracle::ANGUISHED_UNMAKING
};
