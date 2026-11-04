use cantrip_api::*;
use cantrip_mechanics::{choose, effects, select, triggers};
use cantrip_oracle as oracle;

/// "My First Friend — When Zenos yae Galvus enters, choose a creature an opponent controls. Until end of turn, creatures
/// other than Zenos yae Galvus and the chosen creature get -2/-2." The creature is chosen as the ability resolves, without
/// targeting (CR 608.2d), and kept for the linked leaves ability (CR 607.2d). With none to choose, every creature but Zenos
/// still gets -2/-2 (the card's ruling). The affected set is locked in as the effect begins (CR 611.2c).
fn my_first_friend(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let candidates: Vec<ObjectId> = g.state().opponents(you).into_iter().flat_map(|p| g.permanents_of(p)).collect();
    let chosen = choose::select(g, you, item.object, Purpose::Choose, candidates, select::creature_permanents(select::exactly(1)).called("creature an opponent controls"))?;
    if let Some(&creature) = chosen.first() {
        let by = g.linked(item);
        g.remember_chosen_object(by, creature);
    }
    let mut spared = vec![item.object];
    spared.extend(chosen);
    effects::pump_creatures_other_than(g, item.object, &spared, -2, -2);
    Ok(())
}

/// "When the chosen creature leaves the battlefield": the creature My First Friend chose (CR 607.2d), leaving as it last
/// existed (CR 603.10a).
fn chosen_creature_leaves(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, .. } if from.is(ZoneKind::Battlefield) && q.state().chosen_object(q.linked_to(me)) == Some(*old) => {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "... transform Zenos yae Galvus." (CR 701.27f): nothing if Zenos has left or already transformed.
fn transform_zenos(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.transform_by(item.object, item)
}

/// "As this creature transforms into Shinryu, choose an opponent." (CR 712.20, 607.2d): its controller as it turns over
/// chooses, and Burning Chains reads the choice.
fn choose_rival(g: &mut dyn Game, asker: &Asker) -> Result<(), Suspend> {
    let opponents = g.state().opponents(asker.controller).collect();
    if let Some(rival) = choose::player(g, asker.controller, asker.source, opponents, "is the chosen player", 0)? {
        let by = g.linked_to(asker);
        g.remember_chosen_player(by, rival);
    }
    Ok(())
}

/// "Burning Chains — When the chosen player loses the game": a loss looks back (CR 603.10f), so Shinryu triggers even as
/// the rest of that player's game leaves with them (CR 800.4a).
fn chosen_player_loses(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::GameLost { player, .. } if q.state().chosen_player(q.linked_to(me)) == Some(*player) => Fired::PLAIN,
        _ => None,
    }
}

/// "... you win the game." (CR 104.2b.)
fn you_win(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.win_game(item.controller);
    Ok(())
}

/// Flying is a core keyword read from its characteristics.
pub static BACK: CardDef = CardDef {
    as_transforms: Some(choose_rival),
    abilities: &[triggered(
        when_asked(EventMask::GAME_OVER, chosen_player_loses),
        you_win,
        "Burning Chains — When the chosen player loses the game, you win the game.",
    )],
    ..oracle::SHINRYU_TRANSCENDENT_RIVAL
};

pub static CARD: CardDef = CardDef {
    back: Some(&BACK),
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            my_first_friend,
            "My First Friend — When Zenos yae Galvus enters, choose a creature an opponent controls. Until end of turn, creatures other than Zenos yae Galvus and the chosen creature get -2/-2.",
        ),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, chosen_creature_leaves),
            transform_zenos,
            "When the chosen creature leaves the battlefield, transform Zenos yae Galvus.",
        ),
    ],
    ..oracle::ZENOS_YAE_GALVUS
};
