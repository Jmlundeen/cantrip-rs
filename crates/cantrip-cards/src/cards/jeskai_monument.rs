use cantrip_api::*;
use cantrip_mechanics::lands::sub;
use cantrip_mechanics::select::{basic_land_of_types, up_to, Selection};
use cantrip_mechanics::{library, tokens, triggers};
use cantrip_oracle as oracle;

/// "A basic Island, Mountain, or Plains card", which may be left unfound (CR 701.23b).
const BASIC_CARD: Selection =
    Selection::of(up_to(1), &[basic_land_of_types::<{ sub(Subtype::Island) }, { sub(Subtype::Mountain) }, { sub(Subtype::Plains) }>()])
        .called("basic Island, Mountain, or Plains");

/// "When this artifact enters, search your library for a basic Island, Mountain, or Plains card, reveal it, put it into your hand,
/// then shuffle" (CR 701.23).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, BASIC_CARD, true)?;
    Ok(())
}

/// "Create two 1/1 white Bird creature tokens with flying" (CR 111.2).
fn create_tokens(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BIRD, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            search,
            "When this artifact enters, search your library for a basic Island, Mountain, or Plains card, reveal it, put it into your hand, then shuffle.",
        ),
        ability(
            cost("{1}{U}{R}{W}, {T}").and(SACRIFICE_THIS),
            create_tokens,
            "{1}{U}{R}{W}, {T}, Sacrifice this artifact: Create two 1/1 white Bird creature tokens with flying. Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::JESKAI_MONUMENT
};

pub static EXTRA: &[&CardDef] = &[&tokens::BIRD];
