use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ATTACKS, THIS_DIES};
use cantrip_mechanics::{effects, each_discards_a_card, tokens};
use cantrip_oracle as oracle;

/// "Whenever Aclazotz attacks, each opponent discards a card. For each opponent who can't, you draw a card." Each
/// opponent chooses in turn order and the cards are discarded together (CR 101.4, 701.9). An opponent "can't" with no card
/// in hand, or when nothing lets this ability make them discard (CR 101.2: Tamiyo, Collector of Tales). The draws follow
/// the discards, one draw for each such opponent.
fn each_opponent_discards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let opponents: Vec<PlayerId> = g.state().apnap().filter(|&p| p != you).collect();
    let cant = opponents
        .iter()
        .filter(|&&p| g.state().zone(Zone::hand(p)).is_empty() || !may_cause_discard_or_sacrifice(&*g, you, p, false))
        .count();
    each_discards_a_card(g, &opponents, item.object)?;
    for _ in 0..cant {
        g.draw(you, 1);
    }
    Ok(())
}

/// "Whenever an opponent discards a land card" (CR 603.2): each land card discarded is one event, so two discarded at once
/// trigger it twice. The card is read in the zone it went to.
fn opponent_discards_a_land_card(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::Discarded { player, card } if *player != me.controller && q.chars(*card).is(CardType::Land) => Fired::PLAIN,
        _ => None,
    }
}

fn create_bat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BAT, 1)?;
    Ok(())
}

/// "When Aclazotz dies, return it to the battlefield tapped and transformed under its owner's control." The card it became
/// in the graveyard, if it is still there (CR 400.7e); a permanent that is not a double-faced card (a copy of Aclazotz) has
/// no back face and stays in the graveyard (CR 712.14a).
fn return_transformed(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = g.found_after_move(item.object) else { return Ok(()) };
    let owner = g.obj(card).owner;
    g.put_onto_battlefield_transformed(card, owner, true)?;
    Ok(())
}

/// "Activate only if a player has one or fewer cards in hand": any player's hand, an opponent's too.
fn a_player_has_one_or_fewer_cards_in_hand(q: &dyn Query, _: &Asker) -> bool {
    let state = q.state();
    state.apnap().any(|p| state.zone(Zone::hand(p)).len() <= 1)
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::B), "{T}: Add {B}."),
        ability(cost("{2}{B}, {T}"), effects::transform_self, "{2}{B}, {T}: Transform this land. Activate only if a player has one or fewer cards in hand and only as a sorcery.")
            .sorcery_speed()
            .condition(a_player_has_one_or_fewer_cards_in_hand),
    ],
    ..oracle::TEMPLE_OF_THE_DEAD
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS, each_opponent_discards, "Whenever Aclazotz attacks, each opponent discards a card. For each opponent who can't, you draw a card."),
        triggered(when_asked(EventMask::DISCARDED, opponent_discards_a_land_card), create_bat, "Whenever an opponent discards a land card, create a 1/1 black Bat creature token with flying."),
        triggered(THIS_DIES, return_transformed, "When Aclazotz dies, return it to the battlefield tapped and transformed under its owner's control.")
            .look_back(),
    ],
    back: Some(&BACK),
    ..oracle::ACLAZOTZ_DEEPEST_BETRAYAL
};

pub static EXTRA: &[&CardDef] = &[&tokens::BAT];
