use cantrip_api::*;
use cantrip_mechanics::{effects, lands, tokens};
use cantrip_oracle as oracle;

/// "All creatures get -X/-X until end of turn, where X is the number of differently named lands you control. If X is 6 or
/// more, create two ... tokens with haste." X is fixed once as the spell resolves (CR 608.2h). The set count follows the
/// shared name rules for lands (CR 201.2a-b); the token instruction is independent of how many creatures survived.
fn resolve(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = lands::differently_named_lands(g, item.controller) as i32;
    effects::pump_all_creatures(g, item.object, -x, -x);
    if x >= 6 {
        tokens::create(g, item.controller, &tokens::KLINGON, 2)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], resolve),
    ..oracle::THE_DESTRUCTION_OF_PRAXIS
};

pub static EXTRA: &[&CardDef] = &[&tokens::KLINGON];
