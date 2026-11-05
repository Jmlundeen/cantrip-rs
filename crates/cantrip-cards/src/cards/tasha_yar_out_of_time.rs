use cantrip_api::*;
use cantrip_mechanics::library::exile_top_card_play_until_end_of_next_turn;
use cantrip_mechanics::{copy, delayed, triggers};
use cantrip_oracle as oracle;

/// "{2}{R}, Exile this card from your graveyard: Create a token that's a copy of this card. Sacrifice it at the beginning of
/// the next end step." (CR 707.2, 603.7c, 400.7): the copy is of the card as it last was, and the delayed ability is about
/// the token this resolution made, not a later object with its name.
fn make_a_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let tokens = g.create_token_copies(item.object, item.controller, 1)?;
    delayed::at_next_end_step_about(g, item, 0, &tokens);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_DIES,
            exile_top_card_play_until_end_of_next_turn,
            "When Tasha Yar dies, exile the top card of your library. Until the end of your next turn, you may play that card.",
        ),
        ability(
            cost("{2}{R}").and(EXILE_THIS),
            make_a_copy,
            "{2}{R}, Exile this card from your graveyard: Create a token that's a copy of this card. Sacrifice it at the beginning of the next end step. Activate only as a sorcery.",
        )
        .from(ZoneKind::Graveyard)
        .sorcery_speed(),
    ],
    delayed: &[copy::SACRIFICE_IT_AT_END_STEP],
    ..oracle::TASHA_YAR_OUT_OF_TIME
};
