use cantrip_api::*;
use cantrip_mechanics::{effects, lands, triggers};
use cantrip_oracle as oracle;

const GAIN_LIFE: Question = Question::new("GainLife", "gain 1 life?", "", "Gain 1 life", "Don't gain life").amount(1);

/// "Whenever an Island an opponent controls becomes tapped, you may gain 1 life." (CR 608.2, 119.3).
fn gain_1_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if effects::ask_may(g, item.controller, item.object, Purpose::Ask(&GAIN_LIFE))? {
        g.gain_life(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(
                EventMask::TAPPED,
                triggers::subtype_an_opponent_controls_becomes_tapped::<{ lands::sub(Subtype::Island) }>,
            ),
            gain_1_life,
            "Whenever an Island an opponent controls becomes tapped, you may gain 1 life.",
        ),
    ],
    ..oracle::THOUGHTLEECH
};
