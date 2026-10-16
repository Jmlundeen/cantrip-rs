use cantrip_api::*;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

fn pump_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 2, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}{G}"), pump_self, "{1}{G}: This creature gets +2/+2 until end of turn. Activate only once each turn.")
        .once_each_turn()],
    madness: Some(ManaCost::ZERO),
    ..oracle::BASKING_ROOTWALLA
};
