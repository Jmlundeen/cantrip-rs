use cantrip_api::*;
use cantrip_mechanics::{effects, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "When this creature dies, create X 1/1 white Tribble creature tokens with 'This token can't block,' where X is this
/// creature's power" (CR 603.6c, 608.2h, 701.7): its power as it last existed on the battlefield, +1/+1 counters and
/// all, and the tokens enter together as one event.
fn dies(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = targets::power_or_last_known(g, item.object).max(0) as u32;
    tokens::create(g, item.controller, &tokens::TRIBBLE, power)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::BEGINNING_OF_YOUR_COMBAT,
            effects::plus_one_counter_on_this,
            "At the beginning of combat on your turn, put a +1/+1 counter on this creature.",
        ),
        triggered(
            triggers::THIS_DIES,
            dies,
            "When this creature dies, create X 1/1 white Tribble creature tokens with \"This token can't block,\" where X is this creature's power.",
        ),
    ],
    ..oracle::TRIBBLE_CLUSTER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TRIBBLE];
