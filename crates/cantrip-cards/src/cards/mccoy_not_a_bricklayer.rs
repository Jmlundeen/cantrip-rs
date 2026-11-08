use cantrip_api::*;
use cantrip_mechanics::{effects, lands, statics, triggers};
use cantrip_oracle as oracle;

/// "Whenever McCoy or another Doctor you control enters" (CR 603.6a, 201.5): McCoy itself entering, or another Doctor
/// permanent under its controller's control as it entered, the subtype read through the words of McCoy's text (CR 612).
fn mccoy_or_another_doctor_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    triggers::this_or_another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Doctor) }>(q, me, event)
}

/// "Creatures you control with counters on them have lifelink" (CR 613.1f, 702.15a): any counter kind, read afresh as
/// layers apply.
fn creatures_you_control_with_counters(q: &dyn LayerQuery, source: ObjectId, affected: ObjectId, so_far: &Chars) -> bool {
    statics::your_creatures(q, source, affected, so_far) && q.state().obj(affected).has_counters()
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, mccoy_or_another_doctor_enters),
        effects::plus_one_counter_on_each_creature_you_control,
        "Whenever McCoy or another Doctor you control enters, put a +1/+1 counter on each creature you control.",
    )],
    statics: &[static_ability(
        Layer::Ability,
        creatures_you_control_with_counters,
        statics::have_lifelink,
        "Creatures you control with counters on them have lifelink.",
    )],
    ..oracle::MCCOY_NOT_A_BRICKLAYER
};
