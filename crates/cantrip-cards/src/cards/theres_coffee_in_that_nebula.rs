use cantrip_api::*;
use cantrip_mechanics::library::{look_at_top_select, Rest};
use cantrip_mechanics::{beam_me_up, select, targets};
use cantrip_oracle as oracle;

/// "A Plains card or nonland permanent card with mana value 3 or less": either quality, judged of each of the five cards.
/// A Plains card is one with the Plains basic land type, read through this spell's words (CR 612.2); a nonland permanent
/// card is a permanent card that is not a land, its mana value as it is in the library (CR 202.3).
fn plains_or_cheap(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    let plains = q.words(choice.source).subtype(Subtype::Plains);
    Verdict::each(choice, |card| {
        let chars = q.chars(card);
        chars.has_subtype(plains)
            || (targets::is_permanent_card(&chars) && !chars.is(CardType::Land) && q.mana_value(card) <= 3)
    })
}

/// "Look at the top five cards of your library. You may reveal a Plains card or nonland permanent card with mana value 3 or
/// less from among them and put it into your hand. Put the rest on the bottom in a random order."
fn look(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = Selection::new(select::up_to(1))
        .with(select::part(&(plains_or_cheap as SelectFn), 0, ""))
        .called("Plains card or nonland permanent card with mana value 3 or less");
    look_at_top_select(g, item.controller, item.object, 5, rule, true, Rest::BottomRandom)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], look),
    ways: &[beam_me_up::way(mana("{2}{W}"), "Beam me up {2}{W}")],
    ..oracle::THERES_COFFEE_IN_THAT_NEBULA
};
