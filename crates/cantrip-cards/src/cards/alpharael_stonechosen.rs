use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ATTACKS, ward_discard_at_random};
use cantrip_mechanics::{effects, history};
use cantrip_oracle as oracle;

/// Void (intervening "if", CR 603.4): "a nonland permanent left the battlefield this turn or a spell was warped this
/// turn", checked as Alpharael attacks and again as the ability resolves.
fn void(q: &dyn Query, _: &Asker) -> bool {
    history::void(q)
}

/// "Defending player loses half their life, rounded up" (CR 508.5, 107.1a, 119.3): the player Alpharael attacked, or the
/// controller of the planeswalker or protector of the battle it attacked.
fn defending_player_loses_half_their_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(defending) = g.defending_player(item.object) {
        effects::lose_half_their_life(g, defending);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward_discard_at_random("Ward\u{2014}Discard a card at random."),
        triggered(
            THIS_ATTACKS,
            defending_player_loses_half_their_life,
            "Void \u{2014} Whenever Alpharael attacks, if a nonland permanent left the battlefield this turn or a spell was warped this turn, defending player loses half their life, rounded up.",
        )
        .condition(void),
    ],
    ..oracle::ALPHARAEL_STONECHOSEN
};
