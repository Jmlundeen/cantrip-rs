use cantrip_api::*;
use cantrip_mechanics::replicate;
use cantrip_mechanics::statics::{grant_delayed_ability, spell_you_cast};
use cantrip_oracle as oracle;

/// "Artifact spells you cast have replicate {2}{U}." (CR 702.56a, 604.1): while Hugh is on the battlefield each artifact
/// spell its controller casts has the replicate ability granted to it, one instance for each Hugh (legendary, so one), and
/// an artifact spell an opponent casts gets nothing. The spell has it only while it is on the stack, judged by its
/// characteristics there (CR 702.56a: both of replicate's abilities function while the spell is on the stack).
fn artifact_spell_you_cast(view: &dyn LayerQuery, source: ObjectId, affected: ObjectId, chars: &Chars) -> bool {
    chars.is(CardType::Artifact) && spell_you_cast(view, source, affected)
}

pub static CARD: CardDef = CardDef {
    delayed: &[replicate::granted(
        mana("{2}{U}"),
        "Replicate {2}{U} (When you cast this spell, copy it for each time you paid its replicate cost. You may choose new targets for the copies.)",
    )],
    statics: &[static_ability(
        Layer::Ability,
        artifact_spell_you_cast,
        grant_delayed_ability,
        "Artifact spells you cast have replicate {2}{U}.",
    )
    .everywhere()],
    ..oracle::HUGH_FIRST_OF_MANY
};
