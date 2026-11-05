use cantrip_api::*;
use cantrip_mechanics::triggers::this_or_another_of_subtype_you_control_enters;
use cantrip_mechanics::{effects, lands};
use cantrip_oracle as oracle;

/// "Whenever this creature or another Klingon or Warrior you control enters" (CR 603.6a): either subtype, this creature
/// or another, one trigger for the entering permanent.
fn klingon_or_warrior_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    this_or_another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Klingon) }>(q, me, event)
        .or_else(|| this_or_another_of_subtype_you_control_enters::<{ lands::sub(Subtype::Warrior) }>(q, me, event))
}

/// "creatures you control gain lifelink until end of turn" (CR 611.2c, 702.15): each creature its controller controls
/// as the ability resolves.
fn gain_lifelink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::creatures_you_control_gain(g, item, &[Keyword::Lifelink]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, klingon_or_warrior_enters),
        gain_lifelink,
        "Whenever this creature or another Klingon or Warrior you control enters, creatures you control gain lifelink until end of turn.",
    )],
    ..oracle::INDELICATE_HEALER
};
