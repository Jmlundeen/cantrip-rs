use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target Bird, Cat, Dog, Goat, Ox, or Snake" (CR 109.2, 115.1, 205.3m): a permanent with one of those creature types,
/// creature or not, as this land's text names them (CR 612.2).
fn bird_cat_dog_goat_ox_or_snake(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::permanent_of_any_subtype(q, asker, t, &[Subtype::Bird, Subtype::Cat, Subtype::Dog, Subtype::Goat, Subtype::Ox, Subtype::Snake])
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        ability(cost("{2}, {T}"), effects::plus_one_counter_on_target, "{2}, {T}: Put a +1/+1 counter on target Bird, Cat, Dog, Goat, Ox, or Snake.")
        .targets(&[target(bird_cat_dog_goat_ox_or_snake)]),
    ],
    ..oracle::ANIMAL_SANCTUARY
};
