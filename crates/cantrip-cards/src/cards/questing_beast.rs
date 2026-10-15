use cantrip_api::*;
use cantrip_mechanics::damage_target;
use cantrip_oracle as oracle;

/// "Combat damage that would be dealt by creatures you control can't be prevented" (CR 615.12): the damage of any
/// creature its controller controls, the Beast included, and only combat damage.
fn creatures_you_control_deal_unpreventable_combat_damage(q: &dyn Query, asker: &Asker, source: ObjectId, combat: bool) -> bool {
    combat && q.controller(source) == asker.controller && q.chars(source).is(CardType::Creature)
}

/// "Whenever Questing Beast deals combat damage to an opponent": the damage it dealt to that player, which "that much"
/// and "that player controls" read. Zero damage is no damage dealt (CR 120.8).
fn deals_combat_damage_to_an_opponent(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, target: Target::Player(p), amount, combat: true }
            if *from == source && *amount > 0 && *p != q.controller(source) =>
        {
            Some(Fired { subject: Target::Player(*p), data: [*amount as u32, 0] })
        }
        _ => None,
    }
}

/// "Target planeswalker that player controls", the player the damage was dealt to being the event's subject.
fn planeswalker_that_player_controls(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!((t, q.event_subject()), (Target::Object(id), Target::Player(p))
        if q.is_on_battlefield(id) && q.controller(id) == p && q.chars(id).is(CardType::Planeswalker))
}

/// "It deals that much damage to target planeswalker that player controls."
fn damage_the_planeswalker(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    damage_target(g, item, 0, item.data[0] as u16);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    keywords: Keywords::of(&[
        Keyword::Vigilance,
        Keyword::Deathtouch,
        Keyword::Haste,
        Keyword::CantBeBlockedByPowerTwoOrLess,
    ]),
    unpreventable_damage: &[creatures_you_control_deal_unpreventable_combat_damage],
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever Questing Beast deals combat damage to an opponent, it deals that much damage to target planeswalker that player controls.",
        listens: EventMask::DAMAGE,
        matches: deals_combat_damage_to_an_opponent,
        targets: &[TargetSpec { min: 1, max: 1, filter: planeswalker_that_player_controls }],
        effect: damage_the_planeswalker,
        ..TriggeredDef::BLANK
    })],
    ..oracle::QUESTING_BEAST
};
