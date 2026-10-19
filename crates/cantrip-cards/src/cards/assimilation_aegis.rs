use cantrip_api::*;
use cantrip_mechanics::attach::equip;
use cantrip_mechanics::exile_target_until_leaves_effect;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::{THIS_BECOMES_ATTACHED_TO_A_CREATURE, THIS_ENTERS};
use cantrip_oracle as oracle;

/// "For as long as this Equipment remains attached to it" (CR 611.2b): the Equipment is the effect's source, and it is still
/// on the battlefield attached to the creature the effect affects.
fn remains_attached(q: &dyn Query, e: &ContinuousEffect) -> bool {
    q.attached_to(e.source) == Some(e.affected)
}

/// "That creature becomes a copy of a creature card exiled with this Equipment" (CR 707.2, 611.2b, 607.2a). The Equipment is
/// a new object each time it enters, and it exiles at most one card (an "up to one" target), so there is at most one card
/// exiled with it; a creature card is read in exile as it is (a token has ceased to exist, and a card that left is no longer
/// exiled with it). With none the creature is left as it is.
fn become_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(creature) = item.subject else { return Ok(()) };
    let Some(card) = g.exiled_with(item.object).into_iter().find(|&c| g.chars(c).is(CardType::Creature)) else { return Ok(()) };
    let values = g.copiable_values(card);
    g.become_copy(item.object, creature, values, Duration::AsLongAs(g.card_id(&CARD), 0));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            exile_target_until_leaves_effect,
            "When this Equipment enters, exile up to one target creature until this Equipment leaves the battlefield.",
        )
        .targets(&[targets::CREATURE.optional()]),
        triggered(
            THIS_BECOMES_ATTACHED_TO_A_CREATURE,
            become_copy,
            "Whenever this Equipment becomes attached to a creature, for as long as this Equipment remains attached to it, that creature becomes a copy of a creature card exiled with this Equipment.",
        ),
        equip(mana("{2}"), "Equip {2}"),
    ],
    lasting: &[Lasting { text: "{source} remains attached to it", holds: remains_attached }],
    ..oracle::ASSIMILATION_AEGIS
};
