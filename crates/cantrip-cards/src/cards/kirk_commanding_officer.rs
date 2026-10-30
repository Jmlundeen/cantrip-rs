use cantrip_api::*;
use cantrip_mechanics::effects::sacrifice_to_destroy_artifact_or_enchantment;
use cantrip_mechanics::{creatures_have_total_power_at_least, library, triggers};
use cantrip_oracle as oracle;

/// "Whenever you attack while creatures you control have total power 8 or greater": the intervening condition is checked
/// only as the trigger event happens and again as it resolves (CR 508.3d, 603.4).
fn eight_power(q: &dyn Query, asker: &Asker) -> bool {
    creatures_have_total_power_at_least(q, asker.controller, 8)
}

/// "Exile the top card of your library. You may play that card this turn": it is exiled as the trigger resolves, whether
/// or not its controller plays it, and stays playable - cast, or played as a land - until the cleanup step of this turn
/// ends (CR 601.3, 305.1, 514.2). With an empty library nothing is exiled.
fn exile_top_and_allow_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    library::exile_top_play_this_turn(g, item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOU_ATTACK,
            exile_top_and_allow_play,
            "Whenever you attack while creatures you control have total power 8 or greater, exile the top card of your library. You may play that card this turn.",
        )
        .only_if(eight_power),
        sacrifice_to_destroy_artifact_or_enchantment(2, "{2}, Sacrifice this creature: Destroy target artifact or enchantment."),
    ],
    ..oracle::KIRK_COMMANDING_OFFICER
};
