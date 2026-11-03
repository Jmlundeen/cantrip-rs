use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, put a wind counter on this enchantment, then sacrifice this enchantment unless you pay {G} for each wind counter on it. If you pay, this enchantment deals damage equal to the number of wind counters on it to each creature and each player." (CR 608.2c, 120.3, 701.21a).
fn put_wind_counter_enchantment_sacrifice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters_together(&[(item.object, CounterKind::Wind, 1)]);
    g.end_sentence();
    let winds = g.state().obj(item.object).counters(CounterKind::Wind) as u32;
    let mut payment = mana("{G}");
    payment.symbols[Color::Green.mana() as usize] *= winds;
    if g.may_pay(item.controller, item.object, item.object, Cost::mana(payment), &PAY_QUESTION)? {
        g.end_sentence();
        effects::damage_each_creature_and_player_n(g, item, winds);
    } else {
        g.sacrifice(item.controller, &[item.object]);
    }
    Ok(())
}

static PAY_QUESTION: Question = Question::new(
    "Pay",
    "pay {cost}?",
    "If you don't, sacrifice {subject} if you control it",
    "Pay {cost}",
    "Don't pay",
)
.no_default();

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_UPKEEP,
            put_wind_counter_enchantment_sacrifice,
            "At the beginning of your upkeep, put a wind counter on this enchantment, then sacrifice this enchantment unless you pay {G} for each wind counter on it. If you pay, this enchantment deals damage equal to the number of wind counters on it to each creature and each player.",
        ),
    ],
    ..oracle::CYCLONE
};
