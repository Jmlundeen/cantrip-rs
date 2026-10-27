use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOUR_END_STEP, effects::target_player_discards::<2>, "At the beginning of your end step, target opponent discards two cards.")
            .targets(&[targets::OPPONENT]),
        ability(cost("{2}{B}").and(DISCARD_THIS), effects::target_creature_gets::<-4, -4>, "{2}{B}, Discard this card: Target creature gets -4/-4 until end of turn.")
            .targets(&[targets::CREATURE])
            .from(ZoneKind::Hand),
    ],
    ..oracle::ARMUS_SKIN_OF_EVIL
};
