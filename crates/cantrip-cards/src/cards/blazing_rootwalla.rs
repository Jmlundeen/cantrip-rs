use cantrip_api::*;
use cantrip_mechanics::pump;
use cantrip_oracle as oracle;

fn pump_self(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 2, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Activated(ActivatedDef {
        text: "{R}: This creature gets +2/+0 until end of turn. Activate only once each turn.",
        cost: Cost::mana(mana(0, 0, 0, 0, 1, 0)),
        effect: pump_self,
        once_each_turn: true,
        ..ActivatedDef::BLANK
    })],
    madness: Some(ManaCost::ZERO),
    ..oracle::BLAZING_ROOTWALLA
};
