use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Whenever a player casts an enchantment spell" (CR 603.2).
fn when_player_casts_enchantment_spell(
    q: &dyn Query,
    _: &Asker,
    event: &Event,
) -> Option<Fired> {
    cantrip_mechanics::triggers::player_casts_spell_of_type(q, event, CardType::Enchantment)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            when_asked(EventMask::SPELL_CAST, when_player_casts_enchantment_spell),
            cantrip_mechanics::effects::counter_subject_spell,
            "Whenever a player casts an enchantment spell, counter it.",
        ),
    ],
    ..oracle::PRESENCE_OF_THE_MASTER
};
