use cantrip_api::*;
use cantrip_mechanics::statics::{itself, plus_one_per_card_type_in_your_graveyard};
use cantrip_mechanics::{library, select};
use cantrip_oracle as oracle;

/// "Sacrifice a Forest or Plains" (CR 701.21a): a permanent with the subtype Forest or Plains, read through the words of this
/// card's text (CR 612).
fn forest_or_plains(q: &dyn Query, asker: &Asker, id: ObjectId) -> bool {
    let words = q.words(asker.source);
    let chars = q.chars(id);
    chars.has_subtype(words.subtype(Subtype::Forest)) || chars.has_subtype(words.subtype(Subtype::Plains))
}

/// "Search your library for a land card, put it onto the battlefield, then shuffle" (CR 701.23, 701.24).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_battlefield(g, item.controller, item.object, select::LAND_CARD, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::PtModify, itself, plus_one_per_card_type_in_your_graveyard::<{ CardType::Land as u8 }>, "This creature gets +1/+1 for each land card in your graveyard.")],
    abilities: &[ability(
        cost("{T}").and(sacrifice(1, forest_or_plains)),
        search,
        "{T}, Sacrifice a Forest or Plains: Search your library for a land card, put it onto the battlefield, then shuffle.",
    )],
    ..oracle::KNIGHT_OF_THE_RELIQUARY
};
