use cantrip_api::*;
use cantrip_mechanics::history::from_the_future;
use cantrip_mechanics::triggers::YOUR_END_STEP;
use cantrip_mechanics::{damage_target, targets};
use cantrip_oracle as oracle;

/// "If you've played a land or cast a spell this turn from anywhere other than your hand" (CR 603.4, 305.1, 601.2a): an intervening
/// "if", asked as the ability would trigger and again as it resolves. A spell counts by the zone it was cast from (a copy is never
/// cast, CR 707.10), a land by the zone its play moved it from (CR 400.7), whatever has become of either since.
fn played_or_cast_from_outside_your_hand(q: &dyn Query, asker: &Asker) -> bool {
    let you = asker.controller;
    let history = q.history();
    history.casts_by(you).any(|cast| cast.from != ZoneKind::Hand)
        || history.select(EventQuery::kind(EventMask::ZONE_CHANGE).this_turn().reason(ZoneReason::PlayLand)).any(|moved| {
            matches!(moved.event, Event::ZoneChange { new, from, .. } if !from.is(ZoneKind::Hand) && history.any(EventQuery::kind(EventMask::LAND_PLAYED).this_turn().player(you).object(new)))
        })
}

/// "Spider-Man 2099 deals damage equal to his power to any target" (CR 120.3, 608.2h): his power as he last existed if he has left,
/// and none at all if it is 0 or less (CR 120.8). A target no longer legal is dealt nothing (CR 608.2b).
fn damage_equal_to_his_power(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let power = g.chars(item.object).power.and_then(|power| u16::try_from(power).ok()).unwrap_or_default();
    damage_target(g, item, 0, power);
    Ok(())
}

/// Double strike and vigilance are core keywords.
pub static CARD: CardDef = CardDef {
    cast_only_if: Some(from_the_future),
    abilities: &[triggered(YOUR_END_STEP, damage_equal_to_his_power, "At the beginning of your end step, if you've played a land or cast a spell this turn from anywhere other than your hand, Spider-Man 2099 deals damage equal to his power to any target.")
        .targets(&[targets::ANY])
        .condition(played_or_cast_from_outside_your_hand)],
    ..oracle::SPIDER_MAN_2099
};
