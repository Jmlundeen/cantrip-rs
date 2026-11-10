use cantrip_api::*;
use cantrip_mechanics::{assimilate, history, targets};
use cantrip_oracle as oracle;

/// "A creature card from that player's graveyard that was put there from the battlefield this turn" (CR 115.1, 400.7,
/// 404.1): a creature card in the graveyard of the opponent whose slot (`TargetSpec::EACH_OPPONENT`) is being chosen, and
/// one of the cards this turn's moves from the battlefield to that graveyard made - a card milled, discarded or countered
/// was never on the battlefield, and one that left earlier this turn and came back is read by its latest move.
fn creature_card_put_there_from_the_battlefield_this_turn(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!((t, q.slot_opponent()),
        (Target::Object(id), Some(opponent))
            if q.chars(id).is(CardType::Creature)
                && q.obj(id).became.is_none()
                && q.obj(id).zone == Zone::graveyard(opponent)
                && history::put_into_graveyard_from_battlefield_this_turn(q, opponent).contains(&id))
}

const FOR_EACH_OPPONENT: TargetSpec = target(creature_card_put_there_from_the_battlefield_this_turn)
    .between(0, TargetSpec::EACH_OPPONENT)
    .labelled("Creature card {player} lost from the battlefield this turn to assimilate");

/// "For each opponent, assimilate up to one target creature card from that player's graveyard" (CR 608.2f): every target
/// still legal is assimilated as one event, onto the battlefield under this spell's controller's control, each with its
/// +1/+1 counter and as a Borg artifact creature (CR 122.6, 205.1b, 614.12).
fn assimilate_each(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards = targets::legal_objects(g, item);
    assimilate::assimilate_all(g, item, &cards)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[FOR_EACH_OPPONENT], assimilate_each),
    ..oracle::ASSIMILATE_THE_WOUNDED
};
