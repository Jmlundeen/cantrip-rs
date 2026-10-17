use cantrip_api::*;
use cantrip_mechanics::library::exile_top_face_down_may_look_and_play;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target opponent exiles the top X cards of their library face down (CR 406.3). You may look at and play those cards for as
/// long as they remain exiled. If you cast a spell this way, you may spend mana as though it were mana of any type to cast it"
/// (CR 609.4b, 118.14). The cards stay in their owner's exile, and the permission lasts until they leave it. Nothing happens
/// if the target is no longer legal (CR 608.2b). X is the X it was cast with (a copy has none).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    let grant = PlayGrant { lands: true, any_type: true, ..PlayGrant::CAST };
    exile_top_face_down_may_look_and_play(g, opponent, item.x as usize, item.controller, grant);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    ..oracle::OUTRAGEOUS_ROBBERY
};
