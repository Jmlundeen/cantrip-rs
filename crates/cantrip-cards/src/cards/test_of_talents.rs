use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

fn test(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    let controller = g.controller(spell);
    let names = library::object_names(g, spell);
    g.counter(spell);
    library::exile_same_name_from_graveyard_hand_and_library(g, item.controller, controller, item.object, &names)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::INSTANT_OR_SORCERY_SPELL], test),
    ..oracle::TEST_OF_TALENTS
};
