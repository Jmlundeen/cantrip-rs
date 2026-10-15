use cantrip_api::*;
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_mana(item.controller, Mana::B, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect }), ..oracle::DARK_RITUAL };
