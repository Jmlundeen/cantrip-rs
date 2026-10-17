use cantrip_api::*;
use cantrip_mechanics::{pump, spent, triggers};
use cantrip_oracle as oracle;

/// Opus (an ability word, CR 207.2c): "Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If
/// five or more mana was spent to cast that spell, create a token that's a copy of this creature." The trigger resolves before the
/// spell does, and the spell's mana spent (CR 601.2h) is read from it even if it has since been countered or has left the stack
/// (CR 608.2h); a spell cast without paying its mana cost has had none spent. The token is a copy of this creature's copiable
/// values, or its last known ones if it has left (CR 707.2, 608.2h), so it has none of the +1/+1 and has the opus too.
fn opus(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    pump(g, item.object, item.object, 1, 1);
    if spent::five_or_more_mana_spent_on_the_spell(g, item) {
        g.create_token_copies(item.object, item.controller, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::ward::<1>("Ward {1} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {1}.)"),
        triggered(
            triggers::INSTANT_OR_SORCERY_YOU_CAST,
            opus,
            "Opus \u{2014} Whenever you cast an instant or sorcery spell, this creature gets +1/+1 until end of turn. If five or more mana was spent to cast that spell, create a token that's a copy of this creature.",
        ),
    ],
    ..oracle::COLORSTORM_STALLION
};
