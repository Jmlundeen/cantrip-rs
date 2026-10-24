use cantrip_api::*;
use cantrip_mechanics::targets::filters;
use cantrip_mechanics::{choose, effects, targets, tokens, triggers};
use cantrip_oracle as oracle;

/// "You may sacrifice another creature or artifact. If you do, draw a card and create a Treasure token." "Another" is any
/// permanent but this one, and covers both the creature and the artifact. Declining is the empty answer to an up-to-one
/// choice, and nobody is asked when nothing is eligible. "If you do": the permanent chosen is sacrificed (CR 701.21a;
/// nothing stops its controller sacrificing it), and only a chosen one draws and makes the Treasure.
fn sacrifice_for_a_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let eligible: Vec<ObjectId> = g
        .state()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| g.controller(id) == you && filters::another_creature_or_artifact(g, &g.item_asker(item, you), id))
        .collect();
    let chosen = choose::cards_from_kind(g, you, item.object, Purpose::Sacrifice, "other creature or artifact", eligible.clone(), eligible, 1, true)?;
    let Some(&victim) = chosen.first() else { return Ok(()) };
    g.sacrifice(&[victim]);
    effects::draw(g, item, 1)?;
    tokens::create_treasure(g, you)
}

/// "Whenever you sacrifice a token" (CR 701.21a): a token of yours moved from the battlefield by a sacrifice, judged as it
/// was when it left.
fn you_sacrifice_a_token(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let sacrificed = triggers::sacrificed_by(q, me.controller, event)?;
    if q.obj(sacrificed).token { Fired::PLAIN } else { None }
}

/// "Target opponent loses 1 life." (CR 119.3.) A target that is gone loses nothing (CR 608.2b).
fn opponent_loses_a_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(opponent) = targets::player(g, item, 0) {
        g.lose_life(opponent, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, sacrifice_for_a_card, "When The Sackville-Bagginses enter, you may sacrifice another creature or artifact. If you do, draw a card and create a Treasure token."),
        triggered(when_asked(EventMask::ZONE_CHANGE, you_sacrifice_a_token), opponent_loses_a_life, "Whenever you sacrifice a token, target opponent loses 1 life.")
            .targets(&[targets::OPPONENT])
            .on_sacrifice(),
    ],
    ..oracle::THE_SACKVILLE_BAGGINSES
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
