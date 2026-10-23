use cantrip_api::*;
use cantrip_mechanics::crew::crew;
use cantrip_mechanics::triggers::{cycling, THIS_ENTERS};
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "When this Vehicle enters, destroy target nonland permanent an opponent controls with mana value 4 or less."
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, destroy, "When this Vehicle enters, destroy target nonland permanent an opponent controls with mana value 4 or less.")
            .targets(&[target(targets::nonland_permanent_an_opponent_controls_with_mana_value_at_most::<4>)]),
        crew(3, "Crew 3 (Tap any number of other untapped creatures you control with total power 3 or more: This Vehicle becomes an artifact creature until end of turn.)"),
        cycling(mana("{2}"), "Cycling {2} ({2}, Discard this card: Draw a card.)"),
    ],
    ..oracle::THUNDERING_BROODWAGON
};
