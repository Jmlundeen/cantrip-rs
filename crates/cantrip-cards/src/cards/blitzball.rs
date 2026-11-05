use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "Activate only if an opponent was dealt combat damage by a legendary creature this turn" (CR 602.5b, 120.2, 510.2): a combat damage
/// event this turn to an opponent of the activating player, whose source was a legendary creature as it dealt that damage
/// (as the event history keeps it, whatever has become of it since), whoever controlled it. Only creatures deal combat damage (CR 510.2), and prevented or zero damage is never recorded as dealt (CR 615.6, 120.8).
fn opponent_was_dealt_combat_damage_by_a_legendary_creature_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.history().damage_dealt().any(|d| {
        d.combat
            && matches!(d.target, Target::Player(p) if q.state().opponents(asker.controller).any(|opponent| opponent == p))
            && d.chars(q).supertypes.legendary
    })
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color."),
        ability(
            cost("{T}").and(SACRIFICE_THIS),
            |g, item| effects::draw(g, item, 2),
            "GOOOOAAAALLL! \u{2014} {T}, Sacrifice this artifact: Draw two cards. Activate only if an opponent was dealt combat damage by a legendary creature this turn.",
        )
        .condition(opponent_was_dealt_combat_damage_by_a_legendary_creature_this_turn),
    ],
    ..oracle::BLITZBALL
};
