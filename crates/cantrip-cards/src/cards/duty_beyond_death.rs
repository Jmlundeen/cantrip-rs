use cantrip_api::*;
use cantrip_mechanics::effects::{creatures_you_control_gain, plus_one_counter_on_each_creature_you_control};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Creatures you control gain indestructible until end of turn. Put a +1/+1 counter on each creature you control." The
/// creatures are the ones the spell's controller has as it resolves (the sacrificed one, paid as a cost, is gone), and only
/// those (CR 611.2c): the effect and then one counter on each, all together (CR 122.6).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    creatures_you_control_gain(g, item, &[Keyword::Indestructible]);
    plus_one_counter_on_each_creature_you_control(g, item)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    // As an additional cost to cast this spell, sacrifice a creature (CR 118.8, 601.2h): exactly one.
    additional_cost: Some(Cost::sacrifice(1, targets::filters::creature)),
    ..oracle::DUTY_BEYOND_DEATH
};
