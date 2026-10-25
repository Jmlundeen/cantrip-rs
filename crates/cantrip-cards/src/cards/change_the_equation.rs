use cantrip_api::*;
use cantrip_mechanics::{counter_target, targets};
use cantrip_oracle as oracle;

/// "Target red or green spell with mana value 6 or less" (CR 115.1, 105.2, 202.3): the shared red-or-green spell target
/// (Flashfreeze prints it too) whose mana value is 6 or less.
fn red_or_green_spell_up_to_six(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::spell_of_either_color::<{ Color::Red as u8 }, { Color::Green as u8 }>().filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.mana_value(id) <= 6)
}

/// "Counter target spell ..." (CR 701.6), either mode. A spell whose target became illegal is not resolved (CR 608.2b).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(counter, "Counter target spell with mana value 2 or less.").targets(&[target(targets::spell_with_mana_value_at_most::<2>)]),
    mode(counter, "Counter target red or green spell with mana value 6 or less.")
        .targets(&[target(red_or_green_spell_up_to_six)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::CHANGE_THE_EQUATION };
