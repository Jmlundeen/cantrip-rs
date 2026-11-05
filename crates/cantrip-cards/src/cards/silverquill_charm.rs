use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

static MODAL: ModalDef = modes(&[
    mode(effects::two_plus_one_counters_on_target, "Put two +1/+1 counters on target creature.").targets(&[targets::CREATURE]),
    mode(effects::exile_first_target, "Exile target creature with power 2 or less.").targets(&[targets::CREATURE_WITH_POWER_2_OR_LESS]),
    mode(effects::each_opponent_loses_life_you_gain_life::<3>, "Each opponent loses 3 life and you gain 3 life."),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::SILVERQUILL_CHARM };
