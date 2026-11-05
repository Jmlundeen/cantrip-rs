use cantrip_api::*;
use cantrip_mechanics::triggers::{self, THIS_ENTERS_OR_ATTACKS};
use cantrip_mechanics::{each_discards_a_card, each_opponent_sacrifices, effects, hand};
use cantrip_oracle as oracle;

/// "Each player discards a card. Then you draw a card for each card type among cards discarded this way" (CR 701.9,
/// 101.4): everyone chooses, in turn order, and the cards go together; the types are read where the discard put them.
fn discard_then_draw(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    let discarded = each_discards_a_card(g, &players, item.object)?;
    let types = hand::card_types_among_discarded(&*g, &discarded);
    g.draw(item.controller, types as u32);
    Ok(())
}

fn any_permanent(_: &dyn Query, _: ObjectId) -> bool {
    true
}

/// "Each opponent sacrifices a permanent of their choice. Transform Kefka." The transformation happens even when nothing
/// was sacrificed, and does nothing if Kefka has transformed meanwhile (CR 701.27f).
fn sacrifice_and_transform(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    each_opponent_sacrifices(g, item.controller, item.object, 1, any_permanent)?;
    effects::transform_self(g, item)
}

/// "Whenever an opponent loses life during your turn" (CR 119.3, 603.2): the loss of `triggers::opponent_loses_life`, on its
/// controller's turn only.
fn opponent_loses_life_during_your_turn(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    if q.state().turn.active != me.controller {
        return None;
    }
    triggers::opponent_loses_life(q, me, event)
}

/// "You draw that many cards."
fn draw_that_many(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, item.data[0]);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::LIFE_CHANGE, opponent_loses_life_during_your_turn),
        draw_that_many,
        "Whenever an opponent loses life during your turn, you draw that many cards.",
    )],
    ..oracle::KEFKA_RULER_OF_RUIN
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS_OR_ATTACKS, discard_then_draw, "Whenever Kefka enters or attacks, each player discards a card. Then you draw a card for each card type among cards discarded this way."),
        ability(cost("{8}"), sacrifice_and_transform, "{8}: Each opponent sacrifices a permanent of their choice. Transform Kefka. Activate only as a sorcery.").sorcery_speed(),
    ],
    back: Some(&BACK),
    ..oracle::KEFKA_COURT_MAGE
};
