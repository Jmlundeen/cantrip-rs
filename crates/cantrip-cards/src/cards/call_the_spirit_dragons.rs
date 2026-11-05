use cantrip_api::*;
use cantrip_mechanics::{choose, lands, select, statics, triggers::YOUR_UPKEEP};
use cantrip_oracle as oracle;

/// "A Dragon you control of [color]" (CR 105.1, 205.3d): one permanent, none when no Dragon of the color is controlled.
macro_rules! dragon_of {
    ($color:expr) => {
        select::Selection::of(select::as_many_as_can(1), &[select::subtype_card(Subtype::Dragon), select::of_color($color), select::CONTROLLED_BY_YOU]).called("Dragon")
    };
}

/// The selection for each color, in the order of `Color::ALL`.
const DRAGON_OF_COLOR: [select::Selection; 5] =
    [dragon_of!(Color::White), dragon_of!(Color::Blue), dragon_of!(Color::Black), dragon_of!(Color::Red), dragon_of!(Color::Green)];

/// "For each color, put a +1/+1 counter on a Dragon you control of that color. If you put +1/+1 counters on five Dragons this
/// way, you win the game." (CR 105.1, 608.2c, 122.6, 104.2b) The colors go one after another, white, blue, black, red, green:
/// each is its own "put a counter" on a Dragon you control of that color as it stands then, chosen (not targeted) when two or
/// more qualify and skipped when none does. One Dragon of several colors may be chosen for each, and gets a counter every
/// time. The win needs five different Dragons that actually had a counter put on them, so a placement a replacement effect
/// stopped does not count.
fn counter_for_each_color(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let mut received: Vec<ObjectId> = Vec::new();
    for color in Color::ALL {
        // The Dragons are not worked out first: the selection asks for the one, and says which permanents are Dragons of the color.
        let rule = DRAGON_OF_COLOR[color as usize].clone();
        let battlefield = g.state().battlefield().to_vec();
        // No Dragon of the color is controlled: nothing is chosen, so nothing is asked.
        if !cantrip_api::select::first_answer(&*g, &rule, you, item.object, &battlefield, true).is_some_and(|dragons| !dragons.is_empty()) {
            continue;
        }
        let Some(&dragon) = choose::select(g, you, item.object, Purpose::PutCounters(CounterKind::PlusOne, 1), battlefield, rule)?.first() else { continue };
        let before = g.obj(dragon).counters(CounterKind::PlusOne);
        g.add_counters(dragon, CounterKind::PlusOne, 1);
        if g.obj(dragon).counters(CounterKind::PlusOne) > before && !received.contains(&dragon) {
            received.push(dragon);
        }
    }
    if received.len() >= 5 {
        g.win_game(you);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    // "Dragons you control" (CR 613.1f, 205.3d): every permanent its controller controls that the earlier layers made a Dragon,
    // a noncreature one too (a changeling Firdoch Core).
    statics: &[static_ability(Layer::Ability, statics::your_of_subtype::<{ lands::sub(Subtype::Dragon) }>, statics::have_indestructible, "Dragons you control have indestructible.")],
    abilities: &[triggered(
        YOUR_UPKEEP,
        counter_for_each_color,
        "At the beginning of your upkeep, for each color, put a +1/+1 counter on a Dragon you control of that color. If you put +1/+1 counters on five Dragons this way, you win the game.",
    )],
    ..oracle::CALL_THE_SPIRIT_DRAGONS
};
