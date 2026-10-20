use cantrip_api::*;
use cantrip_mechanics::history::gained_life_this_turn;
use cantrip_mechanics::sacrifice_chosen;
use cantrip_mechanics::triggers::{ward_discard, YOUR_END_STEP};
use cantrip_oracle as oracle;

/// "Sacrifice a permanent unless you gained life this turn" (CR 608.2c, 701.21a). The check is made as the ability
/// resolves, not as it triggers (the ruling: it always triggers), and counts life gained by its controller any time this turn
/// (CR 119.3). Otherwise they sacrifice a permanent they control of their choice (this creature among them; a lone permanent is
/// taken without asking).
fn sacrifice_unless_gained_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if gained_life_this_turn(g, item.controller) {
        return Ok(());
    }
    sacrifice_chosen(g, item.controller, item.object, 1, |_| true).map(drop)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ward_discard("Ward\u{2014}Discard a card. (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player discards a card.)"),
        triggered(YOUR_END_STEP, sacrifice_unless_gained_life, "Infusion \u{2014} At the beginning of your end step, sacrifice a permanent unless you gained life this turn."),
    ],
    ..oracle::TRAGEDY_FEASTER
};
