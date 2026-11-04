use cantrip_api::*;
use cantrip_mechanics::{prevention, select};
use cantrip_oracle as oracle;

const BLACK: u8 = ColorSet::of(Color::Black).0;

/// "{1}: The next time a black source of your choice would deal damage to you this turn, prevent that damage." (CR 609.7a, 615.8).
fn next_time_black_source_choice(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let rule = prevention::SOURCE
        .taking(select::as_many_as_can(1))
        .with(select::part(&(prevention::source_of_colors::<BLACK> as SelectFn), 0, "black"));
    prevention::next_time_source_would_damage_you(g, item, 0, rule)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}"),
            next_time_black_source_choice,
            "{1}: The next time a black source of your choice would deal damage to you this turn, prevent that damage.",
        ),
    ],
    granted_preventions: &[
        GrantedPreventionDef {
            hint: "{1}: The next time a black source of your choice would deal damage to you this turn, prevent that damage.",
            damage: "damage",
            of_it: None,
            applies: prevention::from_chosen_source_of_colors::<BLACK>,
            then: None,
            text: "The next time a black source of your choice would deal damage to you this turn, prevent that damage.",
        },
    ],
    ..oracle::CIRCLE_OF_PROTECTION_BLACK
};
