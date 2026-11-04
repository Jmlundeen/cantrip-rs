use cantrip_api::*;
use cantrip_mechanics::untapped_creature_you_control;
use cantrip_oracle as oracle;

/// "Tap two untapped creatures you control: Add one mana of any color." A mana ability (CR 605.1a): the two creatures are chosen
/// with the activation (CR 118.3). They are not the {T} symbol of any creature, so a creature that came under your control this
/// turn can be tapped for it (CR 302.6), and Supportive Parents itself may be one of the two (the cost has no {T} of its own).
pub static CARD: CardDef = CardDef {
    abilities: &[mana_ability(
        Cost::tap_untapped(2, untapped_creature_you_control),
        add_any_color(1),
        "Tap two untapped creatures you control: Add one mana of any color.",
    )],
    ..oracle::SUPPORTIVE_PARENTS
};
