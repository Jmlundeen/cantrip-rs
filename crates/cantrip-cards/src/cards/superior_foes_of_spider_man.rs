use cantrip_api::*;
use cantrip_mechanics::effects::ask_may;
use cantrip_mechanics::library::exile_top_play_until_another_exiled_with;
use cantrip_mechanics::triggers::YOU_CAST_A_SPELL_WITH_MANA_VALUE_4_OR_GREATER;
use cantrip_oracle as oracle;

const ASK: Question = Question::new("ExileTop", "exile the top card of your library?", "Chosen as Superior Foes of Spider-Man's trigger resolves", "Exile it", "Don't");

/// "You may exile the top card of your library. If you do, you may play that card until you exile another card with this
/// creature." (CR 305.1, 601.3, 611.2a): the next card it exiles this way ends the last one's permission, and nothing else
/// does - not this creature leaving the battlefield (`Expiry::UntilAnotherExiledWith`). Declining exiles nothing and so ends
/// nothing.
fn exile_the_top_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if ask_may(g, item.controller, item.object, Purpose::Ask(&ASK))? {
        exile_top_play_until_another_exiled_with(g, item.controller, item.object);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOU_CAST_A_SPELL_WITH_MANA_VALUE_4_OR_GREATER,
        exile_the_top_card,
        "Whenever you cast a spell with mana value 4 or greater, you may exile the top card of your library. If you do, you may play that card until you exile another card with this creature.",
    )],
    ..oracle::SUPERIOR_FOES_OF_SPIDER_MAN
};
