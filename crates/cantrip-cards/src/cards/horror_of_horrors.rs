use cantrip_api::*;
use cantrip_mechanics::{lands, regenerate, targets};
use cantrip_oracle as oracle;

/// Regenerate target black creature (CR 701.19a).
fn regenerate_target_black_creature(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    regenerate::target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                sacrifice(1, targets::filters::land_type::<{ lands::sub(Subtype::Swamp) }>),
                regenerate_target_black_creature,
                "Sacrifice a Swamp: Regenerate target black creature.",
            )
            .targets(&[targets::creature_of_color::<{ Color::Black as u8 }>()]),
    ],
    ..oracle::HORROR_OF_HORRORS
};
