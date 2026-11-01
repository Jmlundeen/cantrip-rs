use cantrip_api::*;
use cantrip_mechanics::effects::sacrifice_to_destroy_artifact_or_enchantment;
use cantrip_mechanics::{creatures_have_total_power_at_least, library, triggers};
use cantrip_oracle as oracle;

/// "Whenever you attack while creatures you control have total power 8 or greater": a condition of the trigger event,
/// judged as the attackers are declared (CR 508.3d, 603.2) - not an intervening "if" (CR 603.4), so it is not judged again
/// as the ability resolves (`only_if`).
fn eight_power(q: &dyn Query, asker: &Asker) -> bool {
    creatures_have_total_power_at_least(q, asker.controller, 8)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOU_ATTACK,
            library::exile_top_card_play_this_turn,
            "Whenever you attack while creatures you control have total power 8 or greater, exile the top card of your library. You may play that card this turn.",
        )
        .only_if(eight_power),
        sacrifice_to_destroy_artifact_or_enchantment(2, "{2}, Sacrifice Kirk: Destroy target artifact or enchantment."),
    ],
    ..oracle::KIRK_COMMANDING_OFFICER
};
