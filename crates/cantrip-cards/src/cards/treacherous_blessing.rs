use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever you cast a spell, you lose 1 life." (CR 119.3).
fn lose_1_life(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.lose_life(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::THIS_ENTERS, |g, item| effects::draw(g, item, 3), "When this enchantment enters, draw three cards."),
        triggered(triggers::YOU_CAST_A_SPELL, lose_1_life, "Whenever you cast a spell, you lose 1 life."),
        triggered(
            triggers::THIS_BECOMES_THE_TARGET_OF_A_SPELL_OR_ABILITY,
            effects::sacrifice_self,
            "When this enchantment becomes the target of a spell or ability, sacrifice it.",
        ),
    ],
    ..oracle::TREACHEROUS_BLESSING
};
