use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever a player casts a white spell, you may pay {1}. If you do, you gain 1 life." (CR 608.2, 119.3, 118.1).
fn pay_1_to_gain_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::may_pay_to_gain_life(g, item, cost("{1}"))
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::SPELL_CAST, triggers::a_player_casts_a_spell_of_color::<{ Color::White as u8 }>),
            pay_1_to_gain_life,
            "Whenever a player casts a white spell, you may pay {1}. If you do, you gain 1 life.",
        ),
    ],
    ..oracle::IVORY_CUP
};
