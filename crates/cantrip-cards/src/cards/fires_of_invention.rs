use cantrip_api::*;
use cantrip_mechanics::lands_you_control;
use cantrip_oracle as oracle;

/// "You can cast spells only during your turn and you can cast no more than two spells each turn. You may cast spells with mana value less than or equal to the number of lands you control without paying their mana costs." (CR 604.1).
fn can_cast_spells_only_during_rule(
    q: &dyn Query,
    asker: &Asker,
    player: PlayerId,
    _: ObjectId,
) -> bool {
    !(player == asker.controller) || q.state().turn.active == player
}

/// "You can cast spells only during your turn and you can cast no more than two spells each turn. You may cast spells with mana value less than or equal to the number of lands you control without paying their mana costs." (CR 604.1).
fn can_cast_spells_only_during_rule_2(
    q: &dyn Query,
    asker: &Asker,
    player: PlayerId,
    spell: ObjectId,
) -> bool {
    !(player == asker.controller
        && q.history().casts_by(player).filter(|c| c.spell != spell).count() >= 2)
}

/// "You can cast spells only during your turn and you can cast no more than two spells each turn. You may cast spells with mana value less than or equal to the number of lands you control without paying their mana costs." (CR 604.1).
fn can_cast_spells_only_during_rule_3(
    q: &dyn Query,
    asker: &Asker,
    spell: ObjectId,
    player: PlayerId,
) -> bool {
    player == asker.controller && q.mana_value(spell) as usize <= lands_you_control(q, player)
}

pub static CARD: CardDef = CardDef {
    spell_alternatives: &[
        SpellAlternative {
            text: "Without paying its mana cost",
            applies: can_cast_spells_only_during_rule_3,
            cost: InsteadOfManaCost::WithoutPaying,
            flash: false,
        },
    ],
    spell_rules: &[can_cast_spells_only_during_rule, can_cast_spells_only_during_rule_2],
    ..oracle::FIRES_OF_INVENTION
};
