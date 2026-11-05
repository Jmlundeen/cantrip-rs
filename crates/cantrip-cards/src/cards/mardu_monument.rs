use cantrip_api::*;
use cantrip_mechanics::lands::sub;
use cantrip_mechanics::select::{basic_land_of_types, up_to, Selection};
use cantrip_mechanics::{library, mobilize, tokens, triggers};
use cantrip_oracle as oracle;

/// "A basic Mountain, Plains, or Swamp card", which may be left unfound (CR 701.23b).
const BASIC_CARD: Selection =
    Selection::of(up_to(1), &[basic_land_of_types::<{ sub(Subtype::Mountain) }, { sub(Subtype::Plains) }, { sub(Subtype::Swamp) }>()])
        .called("basic Mountain, Plains, or Swamp");

/// "When this artifact enters, search your library for a basic Mountain, Plains, or Swamp card, reveal it, put it into your hand,
/// then shuffle" (CR 701.23).
fn search(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, BASIC_CARD, true)?;
    Ok(())
}

/// "Create three 1/1 red Warrior creature tokens. They gain menace and haste until end of turn" (CR 111.2, 611.2a): the tokens
/// made (a token doubler makes more) all gain both keywords for the turn, from the sacrificed Monument.
fn create_warriors(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let made = tokens::create(g, item.controller, &mobilize::WARRIOR, 3)?;
    g.one_instruction(&mut |g| {
        for &id in &made {
            g.add_effect(item.object, id, EffectKind::AddKeywords(Keywords::of(&[Keyword::Menace, Keyword::Haste])), Duration::EndOfTurn);
        }
        Ok(())
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            search,
            "When this artifact enters, search your library for a basic Mountain, Plains, or Swamp card, reveal it, put it into your hand, then shuffle.",
        ),
        ability(
            cost("{2}{R}{W}{B}, {T}").and(SACRIFICE_THIS),
            create_warriors,
            "{2}{R}{W}{B}, {T}, Sacrifice this artifact: Create three 1/1 red Warrior creature tokens. They gain menace and haste until end of turn. Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::MARDU_MONUMENT
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
