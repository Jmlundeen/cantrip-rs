use cantrip_api::*;
use cantrip_mechanics::{effects, library, targets};
use cantrip_oracle as oracle;

/// "Tap target creature. Put three stun counters on it. Exile the top two cards of your library. Choose one of them. Until the end of
/// your next turn, you may play that card": the creature is tapped and gets the counters first, whether or not it was untapped
/// (CR 701.26a, 122.1d). The creature is the spell's only target, so when it is no longer legal the spell does not resolve and no
/// card is exiled (CR 608.2b, the ruling).
/// The chosen card may be a land, played under the ordinary land-play rules (CR 305.1), until the cleanup step of the controller's
/// next turn (CR 514.2).
fn tap_stun_and_impulse(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::tap_and_stun::<3>(g, item)?;
    let grant = PlayGrant { instead: None, lands: true, expiry: Expiry::EndOfYourNextTurn, ..PlayGrant::CAST };
    library::exile_top_choose_one_may_play(g, item.controller, item.object, 2, grant)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], tap_stun_and_impulse), ..oracle::RIVERWHEEL_SWEEP };
