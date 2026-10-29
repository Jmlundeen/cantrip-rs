use cantrip_api::*;
use cantrip_mechanics::{delayed, effects, history, targets, triggers};
use cantrip_oracle as oracle;

/// "Exile target creature card ... Create a token that's a copy of it. That token gains haste. Sacrifice it at the beginning
/// of the next end step." Copy its copiable values without haste (CR 707.2); haste is granted separately by a continuous
/// effect and therefore isn't copied (CR 611.2a, 613.1f). Carry exactly that token to the delayed ability (CR 603.7c). An
/// illegal target does nothing (CR 608.2b).
fn exile_copy_with_haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let exiled = g.exile(card);
    let values = g.copiable_values(exiled);
    let tokens = g.create_token_copies_of(values, item.controller, 1)?;
    instruction(g, |g| {
        for &token in &tokens {
            effects::gain_haste(g, item.object, token);
        }
    });
    g.end_sentence();
    delayed::at_next_end_step_about(g, item, 0, &tokens);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, exile_copy_with_haste, "When this creature enters, exile target creature card with mana value 3 or less from your graveyard. Create a token that's a copy of it. That token gains haste. Sacrifice it at the beginning of the next end step.")
            .targets(&[targets::CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD]),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, history::cards_leave_your_graveyard),
            effects::each_opponent_loses_1_life_you_gain_1,
            "Whenever one or more cards leave your graveyard, each opponent loses 1 life and you gain 1 life.",
        )
        .once_per_batch(),
    ],
    delayed: &[delayed::SACRIFICE_CREATED],
    ..oracle::BLASTONEURON
};
