use cantrip_api::*;
use cantrip_mechanics::eternalize;
use cantrip_oracle as oracle;

/// "Activate only if you control a creature with power 4 or greater."
fn ferocious(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().any(|id| {
        let c = q.chars(id);
        c.is(CardType::Creature) && c.power.is_some_and(|p| p >= 4)
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add(Mana::G), "{T}: Add {G}."),
        mana_ability(TAP, add_n(Mana::G, 4), "Ferocious \u{2014} {T}: Add {G}{G}{G}{G}. Activate only if you control a creature with power 4 or greater.")
            .condition(ferocious),
        eternalize::ability(
            mana(2, 0, 0, 0, 0, 2),
            "Eternalize {2}{G}{G} ({2}{G}{G}, Exile this card from your graveyard: Create a token that's a copy of it, except it's a 4/4 black Zombie Snake Druid with no mana cost. Eternalize only as a sorcery.)",
        ),
    ],
    ..oracle::FANATIC_OF_RHONAS
};
