use cantrip_api::*;
use cantrip_mechanics::effects::enters_with_x_counters;
use cantrip_mechanics::mana::one_of_the_chosen_color_per_charge_counter;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            TAP,
            add_any_color(1).by(one_of_the_chosen_color_per_charge_counter),
            "{T}: Choose a color. Add one mana of that color for each charge counter on this artifact.",
        ),
    ],
    // "This artifact enters with X charge counters on it." (CR 614.1c, 122.6).
    enters: Some(enters_with_x_counters::<{ CounterKind::Charge as usize }>),
    ..oracle::ASTRAL_CORNUCOPIA
};
