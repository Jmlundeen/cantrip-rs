use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            triggers::YOUR_END_STEP,
            effects::plus_one_counter_on_this,
            "At the beginning of your end step, if Katara is tapped, put a +1/+1 counter on her.",
        )
        .condition(triggers::this_is_tapped),
        ability(
            cost("{6}"),
            |g, item| effects::draw(g, item, 1),
            "Waterbend {6}: Draw a card. (While paying a waterbend cost, you can tap your artifacts and creatures to help. Each one pays for {1}.)",
        )
        .waterbend(),
    ],
    ..oracle::KATARA_BENDING_PRODIGY
};
