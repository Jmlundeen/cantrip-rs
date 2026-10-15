use cantrip_api::*;
use cantrip_mechanics::{choose, triggers::this_enters};
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
/// (CR 101.4). You may cast any number of spells from among the nonland cards exiled this way without paying their mana costs:
/// offered one at a time, in the order the controller likes, until they decline or none is castable.
fn exile_and_cast_free(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let mut found: Vec<ObjectId> = Vec::new();
    for player in g.state().apnap().collect::<Vec<_>>() {
        dig(g, player, &mut found);
    }
    loop {
        let castable: Vec<ObjectId> = found
            .iter()
            .copied()
            .filter(|&c| is_live(g, c) && g.obj(c).zone.is(ZoneKind::Exile) && g.could_cast_during_resolution(c))
            .collect();
        let chosen = choose::cards_from(g, item.controller, item.object, Purpose::Cast, found.clone(), castable, 1, true)?;
        let Some(&card) = chosen.first() else { return Ok(()) };
        g.cast_during_resolution(card, Some(ManaCost::ZERO))?;
    }
}

fn give_poison(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Player(p) = item.subject {
        g.give_poison(p, item.data[0]);
    }
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "Whenever Etali deals combat damage to a player, they get that many poison counters.",
        listens: EventMask::DAMAGE,
        matches: cantrip_mechanics::triggers::deals_combat_damage_to_player_subject,
        effect: give_poison,
        ..TriggeredDef::BLANK
    })],
    ..oracle::ETALI_PRIMAL_SICKNESS
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When Etali enters, each player exiles cards from the top of their library until they exile a nonland card. You may cast any number of spells from among the nonland cards exiled this way without paying their mana costs.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: exile_and_cast_free,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Activated(ActivatedDef {
            text: "{9}{G/P}: Transform Etali. Activate only as a sorcery.",
            cost: Cost::mana(ManaCost { generic: 9, phyrexian: [0, 0, 0, 0, 1], ..ManaCost::ZERO }),
            timing: Timing::Sorcery,
            effect: cantrip_mechanics::effects::transform_self,
            ..ActivatedDef::BLANK
        }),
    ],
    back: Some(&BACK),
    ..oracle::ETALI_PRIMAL_CONQUEROR
};
