use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_mechanics::{effects, fading, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(fading::enters::<5>),
    abilities: &[
        ability(remove_counters(CounterKind::Fade, 1), effects::exile_first_target_with_this, "Remove a fade counter from Parallax Wave: Exile target creature.")
            .targets(&[targets::CREATURE]),
        fading::upkeep("At the beginning of your upkeep, remove a fade counter from Parallax Wave. If you can't, sacrifice it."),
        triggered(triggers::THIS_LEAVES, effects::return_exiled_to_battlefield, "When Parallax Wave leaves the battlefield, each player returns to the battlefield all cards they own exiled with it."),
    ],
    ..oracle::PARALLAX_WAVE
};
