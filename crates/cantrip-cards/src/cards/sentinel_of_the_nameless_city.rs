use cantrip_api::*;
use cantrip_mechanics::tokens::create_map;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

fn map(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_map(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS_OR_ATTACKS, map, "Whenever this creature enters or attacks, create a Map token."),
    ],
    ..oracle::SENTINEL_OF_THE_NAMELESS_CITY
};
