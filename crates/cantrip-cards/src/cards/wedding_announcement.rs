use cantrip_api::*;
use cantrip_mechanics::{history::creatures_attacked_with, statics::CREATURES_YOU_CONTROL_GET_PLUS_ONE, tokens, triggers::YOUR_END_STEP};
use cantrip_oracle as oracle;

/// "If you attacked with two or more creatures this turn" is `creatures_attacked_with` for the ability's controller (CR 508.1),
/// a copy's controller being whoever copied it (CR 707.10).
///
/// "At the beginning of your end step, put an invitation counter on this enchantment. If you attacked with two or more
/// creatures this turn, draw a card. Otherwise, create a 1/1 white Human creature token. Then if this enchantment has
/// three or more invitation counters on it, transform it." Transforming is part of this ability (the ruling): the count is
/// read as it resolves, and the counters stay through the transform (CR 712.18).
fn end_step(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::Invitation, 1);
    g.end_sentence();
    if creatures_attacked_with(g, item.controller) >= 2 {
        g.draw(item.controller, 1);
    } else {
        tokens::create(g, item.controller, &tokens::HUMAN, 1)?;
    }
    g.end_sentence();
    if g.obj(item.object).counters(CounterKind::Invitation) >= 3 {
        g.transform_by(item.object, item)?;
    }
    Ok(())
}

/// "Creatures you control get +1/+1." (CR 613.4c)
pub static BACK: CardDef = CardDef { statics: &[CREATURES_YOU_CONTROL_GET_PLUS_ONE], ..oracle::WEDDING_FESTIVITY };

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(YOUR_END_STEP, end_step, "At the beginning of your end step, put an invitation counter on this enchantment. If you attacked with two or more creatures this turn, draw a card. Otherwise, create a 1/1 white Human creature token. Then if this enchantment has three or more invitation counters on it, transform it.")],
    back: Some(&BACK),
    ..oracle::WEDDING_ANNOUNCEMENT
};

pub static EXTRA: &[&CardDef] = &[&tokens::HUMAN];
