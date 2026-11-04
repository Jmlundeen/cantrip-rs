use cantrip_api::*;
use cantrip_mechanics::{effects::plus_one_counter_on_each_other_creature_you_control, triggers::YOUR_END_STEP};
use cantrip_oracle as oracle;

/// An intervening "if you control two or more tapped creatures" (CR 603.4): asked as the end step begins and again as the ability
/// resolves, so it does nothing if they have untapped or died by then.
fn you_control_two_or_more_tapped_creatures(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().filter(|&id| q.obj(id).tapped && q.chars(id).is(CardType::Creature)).count() >= 2
}

/// Dawnstrike Vanguard: lifelink is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_END_STEP,
        plus_one_counter_on_each_other_creature_you_control,
        "At the beginning of your end step, if you control two or more tapped creatures, put a +1/+1 counter on each creature you control other than this creature.",
    )
    .condition(you_control_two_or_more_tapped_creatures)],
    ..oracle::DAWNSTRIKE_VANGUARD
};
