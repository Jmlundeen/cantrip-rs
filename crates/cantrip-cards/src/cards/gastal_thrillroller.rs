use cantrip_api::*;
use cantrip_mechanics::{crew, effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::THIS_ENTERS,
            crew::becomes_a_creature,
            "When this Vehicle enters, it becomes an artifact creature until end of turn.",
        ),
        crew::crew(2, "Crew 2"),
        ability(
                cost("{2}{R}").and(discard_cards(1)),
                effects::return_this_card_with_finality,
                "{2}{R}, Discard a card: Return this card from your graveyard to the battlefield with a finality counter on it. Activate only as a sorcery.",
            )
            .sorcery_speed()
            .from(ZoneKind::Graveyard),
    ],
    ..oracle::GASTAL_THRILLROLLER
};
