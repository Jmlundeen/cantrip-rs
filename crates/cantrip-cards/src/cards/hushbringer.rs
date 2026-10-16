use cantrip_api::*;
use cantrip_mechanics::statics::creatures_entering_dont_trigger;
use cantrip_oracle as oracle;

/// "Creatures ... dying don't cause abilities to trigger." (CR 603.2, 700.4): a creature put into a graveyard from the
/// battlefield, judged as it last existed there (CR 603.10a: a land that had become a creature dies as a creature), triggers
/// no ability - "dies" and "leaves the battlefield" abilities alike. Two triggers are not caused by the dying (the rulings):
/// one whose trigger event is the sacrifice (CR 701.21a, "whenever you sacrifice a creature"), and the dying card's own
/// ability that triggers when it is put into a graveyard from anywhere, which functions from the graveyard it reached.
fn creatures_dying(q: &dyn Query, _: &Asker, event: &Event, triggering: &Triggering) -> bool {
    match *event {
        Event::ZoneChange { old, new, from, to, .. } if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) => {
            q.chars(old).is(CardType::Creature) && !triggering.def.on_sacrifice && triggering.source != new
        }
        _ => false,
    }
}

/// "Creatures entering or dying don't cause abilities to trigger."
pub static CARD: CardDef = CardDef { trigger_rules: &[creatures_entering_dont_trigger, creatures_dying], ..oracle::HUSHBRINGER };
