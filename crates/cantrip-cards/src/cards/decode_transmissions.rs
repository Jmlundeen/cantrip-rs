use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_oracle as oracle;

/// "You draw two cards and lose 2 life. Void - If a nonland permanent left the battlefield this turn or a spell was warped
/// this turn, instead you draw two cards and each opponent loses 2 life": the void condition is read as the spell resolves
/// (CR 608.2c); with it the two cards are drawn and the opponents lose the life instead of the controller.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !history::void(g) {
        g.draw(item.controller, 2);
        g.lose_life(item.controller, 2);
        return Ok(());
    }
    g.draw(item.controller, 2);
    g.each_opponent_loses_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::DECODE_TRANSMISSIONS };
