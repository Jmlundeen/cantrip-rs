use cantrip_api::*;
use cantrip_mechanics::library::find_to_hand;
use cantrip_mechanics::select::{self, Part, Selection};
use cantrip_oracle as oracle;

fn spider_hero(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let words = q.words(choice.source);
    let (spider, hero) = (words.subtype(Subtype::Spider), words.subtype(Subtype::Hero));
    Verdict::each(choice, |card| {
        let chars = q.chars(card);
        chars.has_subtype(spider) && chars.has_subtype(hero)
    })
}

/// "A Spider Hero card" (CR 205.3m): a card with both the subtype Spider and the subtype Hero, as it is in the library, each
/// read through the words of this creature's text (CR 612.2).
const SPIDER_HERO_CARD: Part = select::part(&(spider_hero as SelectFn), 0, "");

/// "Search your library for a Spider Hero card, reveal it, put it into your hand, then shuffle" (CR 701.23, 701.20): a quality
/// to fail at, so none need be found (CR 701.23b); the library is shuffled either way.
fn fateful_bite(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    find_to_hand(g, item.controller, item.object, Selection::of(select::up_to(1), &[SPIDER_HERO_CARD]).called("Spider Hero"), true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{2}").and(SACRIFICE_THIS),
        fateful_bite,
        "Fateful Bite \u{2014} {2}, Sacrifice this creature: Search your library for a Spider Hero card, reveal it, put it into your hand, then shuffle. Activate only as a sorcery.",
    )
    .sorcery_speed()],
    ..oracle::RADIOACTIVE_SPIDER
};
