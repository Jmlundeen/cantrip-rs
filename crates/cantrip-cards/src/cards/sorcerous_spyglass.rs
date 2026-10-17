use cantrip_api::*;
use cantrip_mechanics::{hand, statics};
use cantrip_oracle as oracle;

/// "As this artifact enters, look at an opponent's hand, then choose any card name." (CR 614.12a.)
fn enters(g: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    hand::look_at_an_opponents_hand_then_choose_a_name(g, entering)
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters),
    // "Activated abilities of sources with the chosen name can't be activated unless they're mana abilities." (CR 602.5, 605.1a.)
    activation_rules: &[statics::chosen_name_activations_barred],
    ..oracle::SORCEROUS_SPYGLASS
};
