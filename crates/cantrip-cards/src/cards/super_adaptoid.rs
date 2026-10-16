use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_ATTACKS;
use cantrip_oracle as oracle;

/// "Super-Adaptoid's power is equal to the number of legendary creatures you control" (CR 604.3, layer 7a): a
/// characteristic-defining ability, itself included while it is a legendary creature.
fn power_is_legendary_creatures(q: &dyn LayerQuery, source: ObjectId, chars: &mut Chars) {
    let controller = q.state().obj(source).controller;
    let count = q
        .state()
        .battlefield()
        .iter()
        .filter(|&&id| q.state().obj(id).controller == controller && {
            let c = q.chars(id);
            c.is(CardType::Creature) && c.supertypes.legendary
        })
        .count();
    chars.power = Some(count as i16);
}

/// The keywords it compares, in the order its text names them, each with the counter that gives it (CR 122.1b).
const KINDS: [(Keyword, CounterKind); 11] = [
    (Keyword::Haste, CounterKind::Haste),
    (Keyword::Flying, CounterKind::Flying),
    (Keyword::FirstStrike, CounterKind::FirstStrike),
    (Keyword::DoubleStrike, CounterKind::DoubleStrike),
    (Keyword::Deathtouch, CounterKind::Deathtouch),
    (Keyword::Indestructible, CounterKind::Indestructible),
    (Keyword::Lifelink, CounterKind::Lifelink),
    (Keyword::Menace, CounterKind::Menace),
    (Keyword::Reach, CounterKind::Reach),
    (Keyword::Trample, CounterKind::Trample),
    (Keyword::Vigilance, CounterKind::Vigilance),
];

/// "Choose another target creature. If that creature has haste and Super-Adaptoid doesn't, put a haste counter on
/// Super-Adaptoid. Do the same for flying, first strike, double strike, deathtouch, indestructible, lifelink, menace, reach,
/// trample, and vigilance." Each keyword is judged in turn, as both are then (CR 608.2c). Nothing if the target is no longer
/// legal (CR 608.2b); a Super-Adaptoid that has left the battlefield gets no counters (CR 400.7).
fn adapt(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let me = item.object;
    for (keyword, kind) in KINDS {
        if g.chars(creature).keywords.has(keyword) && !g.chars(me).keywords.has(keyword) {
            g.add_counters(me, kind, 1);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    statics: &[characteristic_defining(Layer::PtDefine, power_is_legendary_creatures, "Super-Adaptoid's power is equal to the number of legendary creatures you control.").lost_pt(0, 2)],
    abilities: &[triggered(THIS_ENTERS_OR_ATTACKS, adapt, "Whenever Super-Adaptoid enters or attacks, choose another target creature. If that creature has haste and Super-Adaptoid doesn't, put a haste counter on Super-Adaptoid. Do the same for flying, first strike, double strike, deathtouch, indestructible, lifelink, menace, reach, trample, and vigilance.")
        .targets(&[targets::OTHER_CREATURE])],
    ..oracle::SUPER_ADAPTOID
};
