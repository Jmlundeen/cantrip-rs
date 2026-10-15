use cantrip_api::*;
use cantrip_mechanics::typecycling;
use cantrip_oracle as oracle;

fn draw_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 3);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[], effect: draw_three }),
    abilities: &[typecycling::ability(Subtype::Island, mana(1, 0, 0, 0, 0, 0), "Islandcycling {1} ({1}, Discard this card: Search your library for an Island card, reveal it, put it into your hand, then shuffle.)")],
    ..oracle::L_RIEN_REVEALED
};
