use cantrip_api::*;
use cantrip_mechanics::{library::exile_top_face_down_may_look_and_play, targets};
use cantrip_oracle as oracle;

/// "Target opponent looks at the top card of your library, then exiles it face down. You look at the top two cards of their
/// library, then exile them face down. For each card exiled this way, the player who exiled it may play it for as long as
/// it remains exiled. Mana of any type can be spent to cast spells this way" (CR 406.3, 601.3, 609.4b): each card is
/// exiled face down and shown only to the player who exiles it (CR 406.3b), and the permission is that player's own,
/// lasting only while the card stays the object it is (CR 400.7), for a card that really reached exile - `PlayGrant::lands` because the text says
/// "play", and `any_type` for its last sentence.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let grant = PlayGrant { lands: true, any_type: true, ..PlayGrant::CAST };
    exile_top_face_down_may_look_and_play(g, item.controller, 1, opponent, grant);
    exile_top_face_down_may_look_and_play(g, opponent, 2, item.controller, grant);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::OPPONENT], effect), ..oracle::VULCAN_MIND_MELD };
