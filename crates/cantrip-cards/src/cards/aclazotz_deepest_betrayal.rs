use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ATTACKS, THIS_DIES};
use cantrip_mechanics::{effects, hand, tokens};
use cantrip_oracle as oracle;

/// "Whenever Aclazotz attacks, each opponent discards a card. For each opponent who can't, you draw a card." The draws follow
/// the discards, one draw for each opponent who couldn't (CR 101.4, 701.9).
fn each_opponent_discards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cant = effects::each_opponent_discards_a_card_and_those_who_cant(g, item)?;
    for _ in cant {
        g.draw(item.controller, 1);
    }
    Ok(())
}

/// "Whenever an opponent discards a land card" (CR 603.2): each land card discarded is one event, so two discarded at once
/// trigger it twice. The card is read in the zone it went to; one put into a hidden zone unrevealed is no land card (CR
/// 701.9c, `hand::discarded_card`).
fn opponent_discards_a_land_card(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match (event, hand::discarded_card(q, event)) {
        (Event::Discarded { player, .. }, Some((_, Some(chars)))) if *player != me.controller && chars.is(CardType::Land) => Fired::PLAIN,
        _ => None,
    }
}

fn create_bat(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create(g, item.controller, &tokens::BAT, 1)?;
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
        triggered(THIS_DIES, effects::return_it_tapped_and_transformed, "When Aclazotz dies, return it to the battlefield tapped and transformed under its owner's control."),
    ],
    back: Some(&BACK),
    ..oracle::ACLAZOTZ_DEEPEST_BETRAYAL
};

pub static EXTRA: &[&CardDef] = &[&tokens::BAT];
