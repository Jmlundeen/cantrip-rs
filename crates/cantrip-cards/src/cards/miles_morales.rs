use cantrip_api::*;
use cantrip_mechanics::{double, effects};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::{THIS_ENTERS, YOU_ATTACK};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on Ultimate Spider-Man. He gains hexproof and becomes colorless until end of turn." (CR 122.1, 702.11b,
/// 105.2c, 611.2a): the counter stays; the hexproof and the loss of color are layer 6 and layer 5 effects of this resolution, and
/// nothing once he has left the battlefield (CR 400.7).
fn camouflage(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.add_counters(item.object, CounterKind::PlusOne, 1);
    g.end_sentence();
    instruction(g, |g| {
        effects::gain_hexproof_until_end_of_turn(g, item.object, item.object);
        g.add_effect(item.object, item.object, EffectKind::SetColors(ColorSet::NONE), Duration::EndOfTurn);
    });
    Ok(())
}

/// "Double the number of each kind of counter on each Spider and legendary creature you control" (CR 701.10e): the creatures
/// are those you control as this resolves, a Spider by the type of its characteristics now, and the number of each kind on each one
/// is read before any is doubled, all the counters put on at once (CR 608.2, 101.4).
fn double_counters_on_spiders_and_legends(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let spider = g.words(item.object).subtype(Subtype::Spider);
    let creatures: Vec<ObjectId> = g
        .permanents_of(item.controller)
        .into_iter()
        .filter(|&id| {
            let chars = g.chars(id);
            chars.is(CardType::Creature) && (chars.supertypes.legendary || chars.has_subtype(spider))
        })
        .collect();
    double::each_kind_of_counter(g, &creatures);
    Ok(())
}

pub static BACK: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{2}"),
            camouflage,
            "Camouflage \u{2014} {2}: Put a +1/+1 counter on Ultimate Spider-Man. He gains hexproof and becomes colorless until end of turn.",
        ),
        triggered(
            YOU_ATTACK,
            double_counters_on_spiders_and_legends,
            "Whenever you attack, double the number of each kind of counter on each Spider and legendary creature you control.",
        ),
    ],
    ..oracle::ULTIMATE_SPIDER_MAN
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            THIS_ENTERS,
            effects::plus_one_counter_on_each_target,
            "When Miles Morales enters, put a +1/+1 counter on each of up to two target creatures.",
        )
        .targets(&[targets::CREATURE.between(0, 2)]),
        ability(cost("{3}{R}{G}{W}"), effects::transform_self, "{3}{R}{G}{W}: Transform Miles Morales. Activate only as a sorcery.").sorcery_speed(),
    ],
    back: Some(&BACK),
    ..oracle::MILES_MORALES
};
