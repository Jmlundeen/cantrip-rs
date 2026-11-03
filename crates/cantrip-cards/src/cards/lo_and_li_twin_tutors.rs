use cantrip_api::select::part;
use cantrip_api::*;
use cantrip_mechanics::select::{up_to, Part, Selection};
use cantrip_mechanics::statics::{have_lifelink, your_of_subtype, your_spells_of_types, ANY_SPELL};
use cantrip_mechanics::{lands, library, triggers};
use cantrip_oracle as oracle;

fn lesson_or_noble(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let lesson = lands::word(q, choice.source, lands::sub(Subtype::Lesson));
    let noble = lands::word(q, choice.source, lands::sub(Subtype::Noble));
    Verdict::each(choice, |card| {
        let chars = q.chars(card);
        chars.has_subtype(lesson) || chars.has_subtype(noble)
    })
}

/// "A Lesson or Noble card" (CR 205.3k, 205.3m): a card with either subtype, the types read through the words of this card's text
/// (CR 612.2) and a card as it is in the library.
const LESSON_OR_NOBLE: Part = part(&(lesson_or_noble as SelectFn), 0, "");

/// One Lesson or Noble card, or none: a search for a quality may find nothing (CR 701.23b).
const LESSON_OR_NOBLE_CARD: Selection = Selection::of(up_to(1), &[LESSON_OR_NOBLE]).called("Lesson or Noble");

/// "When Lo and Li enter, search your library for a Lesson or Noble card, reveal it, put it into your hand, then shuffle."
/// (CR 701.23): the controller's own library and their own hand, and the library is shuffled whether or not a card is found.
fn search_for_a_lesson_or_noble(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::find_to_hand(g, item.controller, item.object, LESSON_OR_NOBLE_CARD, true)?;
    Ok(())
}

/// "Noble creatures you control" (CR 613.1f, 205.3m): a creature under the source's controller's control that is a Noble as the
/// earlier layers made it, Lo and Li herself among them if something makes her one.
fn noble_creatures_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.is(CardType::Creature) && your_of_subtype::<{ lands::sub(Subtype::Noble) }>(q, source, affected, so_far)
}

/// "Lesson spells you control" (CR 601.2i, 205.3k): a spell on the stack, a copy included, that is a Lesson as the earlier
/// layers made it and is controlled by the source's controller.
fn lesson_spells_you_control(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    so_far.has_subtype(Subtype::Lesson) && your_spells_of_types::<ANY_SPELL>(q, source, affected, so_far)
}

const LIFELINK_TEXT: &str = "Noble creatures you control and Lesson spells you control have lifelink.";

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        search_for_a_lesson_or_noble,
        "When Lo and Li enter, search your library for a Lesson or Noble card, reveal it, put it into your hand, then shuffle.",
    )],
    statics: &[
        static_ability(Layer::Ability, noble_creatures_you_control, have_lifelink, LIFELINK_TEXT),
        static_ability(Layer::Ability, lesson_spells_you_control, have_lifelink, LIFELINK_TEXT).everywhere(),
    ],
    ..oracle::LO_AND_LI_TWIN_TUTORS
};
