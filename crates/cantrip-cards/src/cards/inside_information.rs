use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "Exile the top X cards of target opponent's library. You may play those cards this turn. If you cast a spell this way, pay
/// life equal to its mana value rather than pay its mana cost."
///
/// The cards leave together, face up (CR 406.3), and each that reached exile is the caster's to play until the turn ends
/// (CR 611.2): a land as their land play, a spell for life equal to its mana value instead of its mana cost (CR 118.9, 119.4;
/// an X in it is 0, CR 107.3, and no other alternative cost goes with it, CR 118.9a), at the times the rules allow and with
/// any additional costs (rulings 2026-06-29). X of 0 exiles nothing.
fn exile_and_play(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let grant = PlayGrant { lands: true, expiry: Expiry::EndOfTurn, life_for_mana_value: true, ..PlayGrant::CAST };
    library::exile_top_may_play(g, opponent, item.x as usize, item.controller, grant);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::OPPONENT], exile_and_play), ..oracle::INSIDE_INFORMATION };
