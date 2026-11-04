use cantrip_api::*;
use cantrip_mechanics::effects::plus_one_counter_on_this;
use cantrip_mechanics::library::exile_top_card_play_until_end_of_next_turn;
use cantrip_oracle as oracle;

/// "Whenever you attack with two or more Spiders" (CR 508.1m, 603.2): one trigger for the declaration, counting the Spiders
/// among the creatures it declared, the type read through the words of this creature's text (CR 612). This creature need not
/// be one of them (its ruling). A creature put onto the battlefield attacking afterwards is not declared (CR 508.4).
fn you_attack_with_two_spiders(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let Event::AttackersDeclared { player, .. } = event else { return None };
    if *player != me.controller {
        return None;
    }
    let spider = q.words(me.source).subtype(Subtype::Spider);
    let spiders = q.state().attackers.iter().filter(|a| q.chars(a.id).has_subtype(spider)).count();
    if spiders >= 2 { Fired::PLAIN } else { None }
}

/// "Whenever Spinneret and Spiderling deals 4 or more damage" (CR 120.2, 603.2c): the damage it deals at once, to anything,
/// added up across its recipients - combat damage split among blockers and a player is dealt simultaneously (CR 510.2) - and
/// triggering only if that total is at least 4 (`sums_at_least`).
fn this_deals_damage(_: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::Damage { source: from, amount, .. } if *from == source && *amount > 0 => Some(Fired { subject: Target::None, data: [*amount as u32, 0] }),
        _ => None,
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::ATTACKERS_DECLARED, you_attack_with_two_spiders), plus_one_counter_on_this, "Whenever you attack with two or more Spiders, put a +1/+1 counter on Spinneret and Spiderling."),
        triggered(when(EventMask::DAMAGE, this_deals_damage), exile_top_card_play_until_end_of_next_turn, "Whenever Spinneret and Spiderling deals 4 or more damage, exile the top card of your library. Until the end of your next turn, you may play that card.")
            .sums_at_least(4),
    ],
    ..oracle::SPINNERET_AND_SPIDERLING
};
