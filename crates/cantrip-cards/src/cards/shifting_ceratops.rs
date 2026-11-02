use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "{G}: This creature gains your choice of reach, trample, or haste until end of turn." (CR 608.2d, 611.2a).
fn creature_gains_choice_reach_trample(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_your_choice_until_end_of_turn(g, item, &CARD, item.object, &[Keyword::Reach, Keyword::Trample, Keyword::Haste])
}

pub static CARD: CardDef = CardDef {
    protection: ColorSet::of(Color::Blue),
    abilities: &[
        ability(
            cost("{G}"),
            creature_gains_choice_reach_trample,
            "{G}: This creature gains your choice of reach, trample, or haste until end of turn.",
        ),
    ],
    labels: &["Reach", "Trample", "Haste"],
    ..oracle::SHIFTING_CERATOPS
};
