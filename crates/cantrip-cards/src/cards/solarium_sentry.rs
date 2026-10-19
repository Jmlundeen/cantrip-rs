use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::opponent_casts_spell_with_mana_value_at_most;
use cantrip_oracle as oracle;

/// "You gain 2 life."
fn gain_two(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::gain_life(g, item, 2)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(when(EventMask::SPELL_CAST, opponent_casts_spell_with_mana_value_at_most::<2>), gain_two, "Whenever an opponent casts a spell with mana value 2 or less, you gain 2 life.")],
    ..oracle::SOLARIUM_SENTRY
};
