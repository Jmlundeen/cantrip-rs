use cantrip_api::*;
use cantrip_mechanics::cumulative_upkeep;
use cantrip_oracle as oracle;

/// "Green creatures have "Cumulative upkeep {1}."" (CR 604.1).
fn green_creatures_have_cumulative_upkeep_affects(
    q: &dyn LayerQuery,
    source: ObjectId,
    _: ObjectId,
    so_far: &Chars,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    (*so_far).colors.has(q.words(asker.source).color(Color::Green))
        && (*so_far).is(CardType::Creature)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        cumulative_upkeep!(
            cantrip_mechanics::cumulative_upkeep::Payment::Mana(mana("{U}")),
            "Cumulative upkeep {U}"
        ),
    ],
    statics: &[
        static_ability(
            Layer::Ability,
            green_creatures_have_cumulative_upkeep_affects,
            cantrip_mechanics::statics::grant_delayed_ability,
            "Green creatures have \"Cumulative upkeep {1}.\"",
        ),
    ],
    delayed: &[
        cumulative_upkeep!(
            cantrip_mechanics::cumulative_upkeep::Payment::Mana(mana("{1}")),
            "Cumulative upkeep {1}"
        ),
    ],
    ..oracle::BREATH_OF_DREAMS
};
