use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose two —" (CR 700.2, 700.2d): exactly two different modes, carried out in the order printed (CR 608.2c).
static MODAL: ModalDef = modes(&[
    mode(effects::counter_first_target, "Counter target spell.").targets(&[targets::SPELL]),
    mode(effects::return_first_target_to_hand, "Return target permanent to its owner's hand.").targets(&[targets::PERMANENT]),
    mode(effects::tap_each_creature_opponents_control, "Tap all creatures your opponents control."),
    mode(|g, item| effects::draw(g, item, 1), "Draw a card."),
])
.choose(2, 2);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::CRYPTIC_COMMAND };
