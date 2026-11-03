use cantrip_api::*;
use cantrip_mechanics::select::{self, up_to};
use cantrip_mechanics::{earthbend, library, tokens};
use cantrip_oracle as oracle;

/// I: "Create two 1/1 white Ally creature tokens" (CR 111.1, 701.7): both at once, under the Saga's controller.
fn two_allies(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::ALLY, 2)?;
    Ok(())
}

/// "A Mountain or Cave card" (CR 205.3i: both are land types): a card with either subtype as it is in the library, both read through the
/// words of this Saga's text (CR 612.2).
fn mountain_or_cave(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let words = q.words(choice.source);
    let (mountain, cave) = (words.subtype(Subtype::Mountain), words.subtype(Subtype::Cave));
    Verdict::each(choice, |card| {
        let chars = q.chars(card);
        chars.has_subtype(mountain) || chars.has_subtype(cave)
    })
}

const MOUNTAIN_OR_CAVE_CARD: select::Selection = select::Selection::of(up_to(1), &[select::part(&(mountain_or_cave as SelectFn), 0, "")]).called("Mountain or Cave");

/// II: "Search your library for a Mountain or Cave card, reveal it, put it into your hand, then shuffle" (CR 701.23), which may
/// be left unfound (CR 701.23b).
fn search_mountain_or_cave(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, MOUNTAIN_OR_CAVE_CARD, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        chapter(1, two_allies, "I \u{2014} Create two 1/1 white Ally creature tokens."),
        chapter(2, search_mountain_or_cave, "II \u{2014} Search your library for a Mountain or Cave card, reveal it, put it into your hand, then shuffle."),
        chapter(3, earthbend::effect::<3>, "III \u{2014} Earthbend 3. (Target land you control becomes a 0/0 creature with haste that's still a land. Put three +1/+1 counters on it. When it dies or is exiled, return it to the battlefield tapped.)")
            .targets(&[earthbend::TARGET]),
    ],
    delayed: &[earthbend::RETURN],
    ..oracle::THE_CAVE_OF_TWO_LOVERS
};

pub static EXTRA: &[&CardDef] = &[&tokens::ALLY];
