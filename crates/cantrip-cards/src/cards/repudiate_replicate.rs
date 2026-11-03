use cantrip_api::*;
use cantrip_mechanics::{copy, counter_target_ability, targets};
use cantrip_oracle as oracle;

/// "Counter target activated or triggered ability." (CR 701.6a, 115.1, 608.2b).
fn counter_target_activated_or_triggered_ability(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target_ability(g, item, 0);
    Ok(())
}

pub static REPUDIATE: CardDef = CardDef {
    spell: spell(&[targets::ACTIVATED_OR_TRIGGERED_ABILITY], counter_target_activated_or_triggered_ability),
    ..oracle::REPUDIATE
};

/// "Create a token that's a copy of target creature you control." (CR 111.1, 707.2).
pub static REPLICATE: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], copy::create_token_copy_of_target),
    ..oracle::REPLICATE
};

/// Repudiate // Replicate: a split card (CR 709.1, 709.3), cast as either half.
pub static CARD: CardDef = CardDef { split: Some((&REPUDIATE, &REPLICATE)), ..oracle::REPUDIATE_REPLICATE };

pub static EXTRA: &[&CardDef] = &[&REPUDIATE, &REPLICATE];
