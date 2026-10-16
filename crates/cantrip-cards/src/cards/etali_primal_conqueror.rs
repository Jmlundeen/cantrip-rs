use cantrip_api::*;
use cantrip_mechanics::{choose, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// One player's dig: exile from the top, one card at a time, until a nonland card is exiled or the library runs out. A card a
/// replacement effect sends elsewhere was not exiled, so the dig goes on.
fn dig(g: &mut dyn Game, player: PlayerId, found: &mut Vec<ObjectId>) {
    while let Some(&top) = g.state().zone(Zone::library(player)).first() {
        let exiled = g.exile(top);
        if is_live(g, exiled) && g.obj(exiled).zone.is(ZoneKind::Exile) && !g.chars(exiled).is(CardType::Land) {
            found.push(exiled);
            return;
        }
    }
}

/// Each player exiles cards from the top of their library until they exile a nonland card, from the active player on
/// (CR 101.4). You may cast any number of spells from among the nonland cards exiled this way without paying their mana costs.
fn exile_and_cast_free(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut found: Vec<ObjectId> = Vec::new();
    for player in g.state().apnap().collect::<Vec<_>>() {
        dig(g, player, &mut found);
    }
    choose::cast_any_number_free(g, item.controller, item.object, &found)
}

fn give_poison(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(p) = item.subject {
        g.give_poison(p, item.data[0]);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(cantrip_mechanics::triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER_SUBJECT, give_poison, "Whenever Etali deals combat damage to a player, they get that many poison counters.")],
    ..oracle::ETALI_PRIMAL_SICKNESS
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_and_cast_free, "When Etali enters, each player exiles cards from the top of their library until they exile a nonland card. You may cast any number of spells from among the nonland cards exiled this way without paying their mana costs."),
        ability(cost("{9}{G/P}"), cantrip_mechanics::effects::transform_self, "{9}{G/P}: Transform Etali. Activate only as a sorcery.")
            .sorcery_speed(),
    ],
    back: Some(&BACK),
    ..oracle::ETALI_PRIMAL_CONQUEROR
};
