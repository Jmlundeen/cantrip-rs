use cantrip_api::*;
use cantrip_api::select::part;
use cantrip_mechanics::{effects, history, library, select, triggers};
use cantrip_oracle as oracle;

fn another_enchantment(q: &dyn Query, choice: &Choice, _: u32) -> Verdict {
    Verdict::each(choice, |id| id != choice.source && q.chars(id).is(CardType::Enchantment))
}

/// "Another enchantment" you control, for the sacrifice (CR 701.21a): an enchantment permanent other than this one.
const ANOTHER_ENCHANTMENT: Selection = Selection::of(select::up_to(1), &[part(&(another_enchantment as SelectFn), 0, "")]).called("other enchantment");

/// "At the beginning of your end step, you may sacrifice another enchantment. If you do, search your library for a creature card
/// with mana value equal to 1 plus the sacrificed enchantment's mana value, put that card onto the battlefield, then shuffle."
/// (CR 608.2c, 701.21a, 701.23).
fn sacrifice_another_enchantment_if_do(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(&sacrificed) = effects::may_sacrifice(g, item.controller, item.object, ANOTHER_ENCHANTMENT)?.first() else { return Ok(()) };
    g.end_sentence();
    let value = history::mana_value_of_sacrificed_permanent(g, sacrificed) + 1;
    library::find_to_battlefield(g, item.controller, item.object, select::CREATURE_CARD.with(select::mana_value_exactly(value)), false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_END_STEP,
            sacrifice_another_enchantment_if_do,
            "At the beginning of your end step, you may sacrifice another enchantment. If you do, search your library for a creature card with mana value equal to 1 plus the sacrificed enchantment's mana value, put that card onto the battlefield, then shuffle.",
        ),
    ],
    ..oracle::ENIGMATIC_INCARNATION
};
