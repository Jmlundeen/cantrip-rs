use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, targets};
use cantrip_oracle as oracle;

/// "Target player discards two cards. Then exile that player's graveyard." The player chooses the two cards and they are
/// discarded together (CR 701.9); then every card in their graveyard, the ones just discarded included (a card a replacement
/// effect sent elsewhere is not there), is exiled at once (CR 406.2, 608.2c). Nothing happens if the target is illegal
/// (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    discard_chosen(g, player, item.object, 2)?;
    let cards = g.state().zone(Zone::graveyard(player)).to_vec();
    g.exile_all(&cards);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::PLAYER], effect), ..oracle::GO_BLANK };
