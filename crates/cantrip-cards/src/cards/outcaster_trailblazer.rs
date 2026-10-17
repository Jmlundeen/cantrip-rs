use cantrip_api::*;
use cantrip_mechanics::plot::plot;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever another creature you control with power 4 or greater enters" (CR 603.6a): a creature other than this one, under
/// your control as it entered, whose power is 4 or more as it is on the battlefield - the static abilities that apply to it, the
/// counters it entered with and what it entered as a copy of all count (the ruling), since the trigger looks at it once it is there.
/// Once triggered, lowering its power or removing it does not stop the draw.
fn another_creature_with_power_four_enters(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    let fired = triggers::another_creature_you_control_enters(q, me, event)?;
    let Target::Object(creature) = fired.subject else { return None };
    (q.chars(creature).power.unwrap_or(0) >= 4).then_some(fired)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, effects::add_one_mana_of_any_color, "When this creature enters, add one mana of any color."),
        triggered(
            when_asked(EventMask::ZONE_CHANGE, another_creature_with_power_four_enters),
            |g, item| effects::draw(g, item, 1),
            "Whenever another creature you control with power 4 or greater enters, draw a card.",
        ),
        plot(
            mana("{2}{G}"),
            "Plot {2}{G} (You may pay {2}{G} and exile this card from your hand. Cast it as a sorcery on a later turn without paying its mana cost. Plot only as a sorcery.)",
        ),
    ],
    ..oracle::OUTCASTER_TRAILBLAZER
};
