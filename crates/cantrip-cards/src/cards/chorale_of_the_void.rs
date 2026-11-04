use cantrip_api::*;
use cantrip_mechanics::combat::choose_attack_target;
use cantrip_mechanics::triggers::{ATTACHED_CREATURE_ATTACKS_SUBJECT, YOUR_END_STEP};
use cantrip_mechanics::{history, targets};
use cantrip_oracle as oracle;

/// "Target creature card from defending player's graveyard" (CR 115.1, 508.5): the defending player is the one the enchanted
/// creature (the trigger's subject) attacks - still, once it has left the battlefield (`Query::defending_player`) - and the
/// card is still that card (CR 400.7).
fn creature_card_in_defending_players_graveyard(q: &dyn Query, _: &Asker, t: Target) -> bool {
    let (Target::Object(id), Target::Object(attacker)) = (t, q.event_subject()) else { return false };
    targets::card_in_graveyard(q, t, &[CardType::Creature]) && q.defending_player(attacker) == Some(q.obj(id).zone.player)
}

/// "Put target creature card from defending player's graveyard onto the battlefield under your control tapped and attacking"
/// (CR 508.4): a new object under your control, attacking a player or planeswalker you choose, which need not be what the
/// enchanted creature attacks; it was never declared as an attacker. Nothing happens if the card has left the graveyard
/// (CR 608.2b).
fn reanimate_attacking(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(card) = targets::object(g, item, 0) else { return Ok(()) };
    let you = item.controller;
    let defender = choose_attack_target(g, you, item.object)?.unwrap_or(Target::None);
    g.put_onto_battlefield_attacking(card, you, defender, &[])?;
    Ok(())
}

/// "Sacrifice this Aura unless a nonland permanent left the battlefield this turn or a spell was warped this turn." Void is an
/// ability word (CR 207.2c); this is no intervening "if", so the ability goes on the stack and the check is made as it resolves.
fn sacrifice_unless_void(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !history::void(g) {
        g.sacrifice(item.controller, &[item.object]);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE_YOU_CONTROL),
    abilities: &[
        triggered(
            ATTACHED_CREATURE_ATTACKS_SUBJECT, // "Whenever enchanted creature attacks" (CR 508.1m)
            reanimate_attacking,
            "Whenever enchanted creature attacks, put target creature card from defending player's graveyard onto the battlefield under your control tapped and attacking.",
        )
        .targets(&[target(creature_card_in_defending_players_graveyard)]),
        triggered(
            YOUR_END_STEP,
            sacrifice_unless_void,
            "Void \u{2014} At the beginning of your end step, sacrifice this Aura unless a nonland permanent left the battlefield this turn or a spell was warped this turn.",
        ),
    ],
    ..oracle::CHORALE_OF_THE_VOID
};
