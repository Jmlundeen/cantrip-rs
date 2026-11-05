use cantrip_api::*;
use cantrip_mechanics::lands::sub;
use cantrip_mechanics::select::{basic_land_of_types, up_to, Selection};
use cantrip_mechanics::{library, tokens, triggers};
use cantrip_oracle as oracle;

/// "A basic Swamp, Forest, or Island card", which may be left unfound (CR 701.23b).
const BASIC_CARD: Selection =
    Selection::of(up_to(1), &[basic_land_of_types::<{ sub(Subtype::Swamp) }, { sub(Subtype::Forest) }, { sub(Subtype::Island) }>()])
        .called("basic Swamp, Forest, or Island");

/// "When this artifact enters, search your library for a basic Swamp, Forest, or Island card, reveal it, put it into your hand,
/// then shuffle" (CR 701.23).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, BASIC_CARD, true)?;
    Ok(())
}

/// "Create two 2/2 black Zombie Druid creature tokens" (CR 111.2).
fn create_tokens(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ZOMBIE_DRUID, 2)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            search,
            "When this artifact enters, search your library for a basic Swamp, Forest, or Island card, reveal it, put it into your hand, then shuffle.",
        ),
        ability(
            cost("{2}{B}{G}{U}, {T}").and(SACRIFICE_THIS),
            create_tokens,
            "{2}{B}{G}{U}, {T}, Sacrifice this artifact: Create two 2/2 black Zombie Druid creature tokens. Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::SULTAI_MONUMENT
};

pub static EXTRA: &[&CardDef] = &[&tokens::ZOMBIE_DRUID];
