use cantrip_api::*;
use cantrip_mechanics::{effects, firebending, trample_until_end_of_turn};
use cantrip_oracle as oracle;

/// "Put two +1/+1 counters on this creature. It gains trample until end of turn." (CR 122.1, 611.2, 702.19): both done to the
/// ability's source, and only if it is still on the battlefield (CR 400.7).
fn two_counters_and_trample(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::two_plus_one_counters_on_this(g, item)?;
    trample_until_end_of_turn(g, item.object, item.object);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        firebending::trigger::<2>("Firebending 2 (Whenever this creature attacks, add {R}{R}. This mana lasts until end of combat.)"),
        ability(cost("{8}"), two_counters_and_trample, "Exhaust \u{2014} {8}: Put two +1/+1 counters on this creature. It gains trample until end of turn. (Activate each exhaust ability only once.)")
            .exhaust(),
    ],
    ..oracle::ROUGH_RHINO_CAVALRY
};
