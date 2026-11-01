use cantrip_api::*;
use cantrip_mechanics::{choose, damage_each_target, library, select, targets, top_of_library};
use cantrip_oracle as oracle;

/// "Up to one target creature that player or that planeswalker's controller controls" (CR 115.1, 601.2c): a creature controlled
/// by the player the first slot chose, or by the controller of the planeswalker it chose.
fn creature_controlled_by_first_target_or_its_controller(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    let controlled_by = |p: PlayerId| matches!(t, Target::Object(id) if q.controller(id) == p);
    let first_controls = |&first: &Target| matches!(first, Target::Player(p) if controlled_by(p)) || matches!(first, Target::Object(walker) if controlled_by(q.controller(walker)));
    (targets::CREATURE.filter)(q, asker, t) && q.chosen_targets().first().is_some_and(first_controls)
}

/// "+1: Chandra deals 1 damage to target player or planeswalker and 1 damage to up to one target creature that player or that
/// planeswalker's controller controls. That creature can't block this turn." The damage is dealt to each target still legal
/// at once (CR 120.3, 608.2b). The second sentence is the creature targeted, if it is still legal as this resolves, and
/// applies even when the damage was prevented or redirected, or the player target became illegal (CR 608.2b).
fn pressure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let creature = targets::object(g, item, 1);
    damage_each_target(g, item, 1);
    g.end_sentence();
    if let Some(creature) = creature {
        g.add_effect(item.object, creature, EffectKind::CantBlock, Duration::EndOfTurn);
    }
    Ok(())
}

/// "−7: Exile the top ten cards of your library. Choose an instant or sorcery card exiled this way and copy it three times. You
/// may cast the copies without paying their mana costs." The cards leave together (CR 701.13); "exiled this way" is those
/// that reached exile now. The copies are of the chosen card in exile (CR 707.12) and each is cast while this resolves,
/// whatever its timing (CR 608.2g); the ones not cast cease to exist (CR 707.10a, 704.5e). Costs other than the mana cost
/// are paid and X is 0 (CR 601.2b, 107.3b). With no instant or sorcery card exiled, nothing is chosen or copied.
fn ultimate(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let top = top_of_library(g, you, 10);
    let exiled: Vec<ObjectId> = g.exile_all(&top).into_iter().filter(|&c| g.obj(c).zone.is(ZoneKind::Exile)).collect();
    let eligible: Vec<ObjectId> = exiled.iter().copied().filter(|&c| library::instant_or_sorcery_card(g, c)).collect();
    let rule = Selection::listed(select::as_many_as_can(1), &exiled, eligible).called("instant or sorcery");
    let chosen = choose::select(g, you, item.object, Purpose::Choose, exiled, rule)?;
    if let Some(&card) = chosen.first() {
        g.end_sentence();
        let copies: Vec<ObjectId> = (0..3).map(|_| g.create_copy_in_exile(card, you)).collect();
        choose::cast_any_number_free(g, you, item.object, &copies)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(1), pressure, "+1: Chandra deals 1 damage to target player or planeswalker and 1 damage to up to one target creature that player or that planeswalker's controller controls. That creature can't block this turn.")
            .targets(&[targets::PLAYER_OR_PLANESWALKER, up_to_one(creature_controlled_by_first_target_or_its_controller)]),
        ability(loyalty(0), library::exile_top_card_play_this_turn, "0: Exile the top card of your library. You may play it this turn."),
        ability(loyalty(-7), ultimate, "\u{2212}7: Exile the top ten cards of your library. Choose an instant or sorcery card exiled this way and copy it three times. You may cast the copies without paying their mana costs."),
    ],
    ..oracle::CHANDRA_PYROMASTER
};
