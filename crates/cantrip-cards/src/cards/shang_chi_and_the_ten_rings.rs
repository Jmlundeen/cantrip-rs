use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "Whenever you draw a card, put a +1/+1 counter on Shang-Chi." (CR 122.6) Only on the Shang-Chi that triggered, if it
/// is still there (CR 400.7).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    Ok(())
}

/// "When the tenth +1/+1 counter is put on Shang-Chi, you draw five cards and gain 5 life."
fn ten_rings(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 5);
    g.gain_life(item.controller, 5);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOU_DRAW_A_CARD, grow, "Whenever you draw a card, put a +1/+1 counter on Shang-Chi."),
        triggered(
            when_asked(EventMask::COUNTERS, triggers::nth_counter_put_on_this::<{ CounterKind::PlusOne as u8 }, 10>),
            ten_rings,
            "When the tenth +1/+1 counter is put on Shang-Chi, you draw five cards and gain 5 life.",
        ),
    ],
    ..oracle::SHANG_CHI_AND_THE_TEN_RINGS
};
