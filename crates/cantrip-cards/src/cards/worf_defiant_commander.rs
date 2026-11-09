use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ENTERS, YOU_ATTACK};
use cantrip_mechanics::{battlefront, library::exile_top_play_until_end_of_next_turn, tokens};
use cantrip_oracle as oracle;

/// "When Worf enters, create a 2/1 red Klingon Warrior creature token with haste." (CR 603.6a, 701.7): one of the shared
/// KLINGON token definition under its controller, exactly as Cha'DIch Investigator makes.
fn create_token(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::KLINGON, 1)?;
    Ok(())
}

/// "Battlefront — Whenever you attack, exile the top X cards of your library, where X is the number of opponents you
/// attacked this turn. Until the end of your next turn, you may play those cards." (CR 508.1): the shared Battlefront
/// count of opponents attacked this turn, exiled and playable through the shared next-turn helper.
fn battlefront_exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let n = battlefront::opponents_attacked_this_turn(g, item.controller);
    exile_top_play_until_end_of_next_turn(g, item.controller, n as usize);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, create_token, "When Worf enters, create a 2/1 red Klingon Warrior creature token with haste."),
        triggered(YOU_ATTACK, battlefront_exile, "Battlefront — Whenever you attack, exile the top X cards of your library, where X is the number of opponents you attacked this turn. Until the end of your next turn, you may play those cards."),
    ],
    ..oracle::WORF_DEFIANT_COMMANDER
};

pub static EXTRA: &[&CardDef] = &[&tokens::KLINGON];
